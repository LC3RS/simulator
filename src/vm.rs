use byteorder::{BigEndian, ReadBytesExt};
use num_traits::{FromPrimitive, ToPrimitive};
use std::{
    fs::File,
    io::{self, BufReader, Read, Write},
    path::PathBuf,
};

use crate::{
    constants::MAX_MEMORY,
    enums::{CondFlag, RawOpCode, Register, TrapCode},
    memory::{MemoryManager, RegisterManager},
    utils::sign_extend,
};

#[derive(Default)]
pub struct Machine {
    reg: RegisterManager,
    mem: MemoryManager,
    is_running: bool,
    debug_mode: bool,
}

impl Machine {
    pub fn enter_debug_mode(&mut self) {
        self.debug_mode = true;
    }

    pub fn debug(&self, s: &str) {
        if self.debug_mode {
            println!("[Debug] {s}");
        }
    }

    pub fn run(&mut self) {
        self.is_running = true;

        while self.is_running && (self.reg.get(Register::PC) as usize) < MAX_MEMORY {
            let raw_instr = self.fetch();
            self.decode_and_execute(raw_instr);
        }
    }

    pub fn load_image(&mut self, path: PathBuf) -> Result<(), io::Error> {
        self.debug(format!("Attempting to load image file: {}", path.display()).as_str());

        let mut file = BufReader::new(File::open(path)?);
        let origin = file.read_u16::<BigEndian>()?;
        let mut addr = origin;

        loop {
            match file.read_u16::<BigEndian>() {
                Ok(instr) => {
                    self.mem.write(addr, instr);
                    addr += 1;
                }
                Err(e) => {
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        self.debug("Image loaded successfully")
                    } else {
                        return Err(e);
                    }
                    break;
                }
            }
        }

        Ok(())
    }

    fn fetch(&mut self) -> u16 {
        let instr = self.mem.read(self.reg.get(Register::PC));
        self.reg.incr(Register::PC);
        instr
    }

    fn decode_and_execute(&mut self, raw_instr: u16) {
        if raw_instr == 0 {
            return;
        }
        self.debug(format!("Instr: {:#018b}", raw_instr).as_str());
        let raw_op = RawOpCode::from_u16(raw_instr >> 12).unwrap();

        match raw_op {
            RawOpCode::Add => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let src1 = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();

                // Check if we are in immediate mode
                let imm_flag = (raw_instr >> 5) & 0x1;

                if imm_flag == 1 {
                    let imm5 = sign_extend(raw_instr & 0x1F, 5);
                    self.reg.set(dest, self.reg.get(src1) + imm5);
                } else {
                    let src2 = Register::from_u16(raw_instr & 0x7).unwrap();
                    self.reg.set(dest, self.reg.get(src1) + self.reg.get(src2));
                }

                self.update_flags(dest);
            }

            RawOpCode::And => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let src1 = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();

                // Check if we are in immediate mode
                let imm_flag = (raw_instr >> 5) & 0x1;

                if imm_flag == 1 {
                    let imm5 = sign_extend(raw_instr & 0x1F, 5);
                    self.reg.set(dest, self.reg.get(src1) & imm5);
                } else {
                    let src2 = Register::from_u16(raw_instr & 0x7).unwrap();
                    self.reg.set(dest, self.reg.get(src1) & self.reg.get(src2));
                }

                self.update_flags(dest);
            }

            RawOpCode::Not => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let src = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();

                self.reg.set(dest, !self.reg.get(src));

                self.update_flags(dest);
            }

            RawOpCode::Br => {
                let cond_flag = (raw_instr >> 9) & 0x7;
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);

                if (cond_flag & self.reg.get(Register::COND)) != 0 {
                    self.reg.incr_by(Register::PC, pc_offset);
                }
            }

            RawOpCode::Jmp => {
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                self.reg.copy(Register::PC, base);
            }

            RawOpCode::Jsr => {
                // Check if instruction was JSR or JSRR
                let miku_bit = (raw_instr >> 11) & 0x1;

                self.reg.copy(Register::R7, Register::PC);

                if miku_bit == 1 {
                    /* JSR */
                    let pc_offset = sign_extend(raw_instr & 0x7FF, 11);
                    self.reg.incr_by(Register::PC, pc_offset);
                } else {
                    /* JSRR */
                    let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                    self.reg.copy(Register::PC, base);
                }
            }

            RawOpCode::Ld => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let addr = self.reg.get(Register::PC) + pc_offset;

                self.reg.set(dest, self.mem.read(addr));
                self.update_flags(dest);
            }

            RawOpCode::Ldr => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                let offset = sign_extend(raw_instr & 0x3F, 6);
                let data = self.mem.read(self.reg.get(base) + offset);

                self.reg.set(dest, data);
                self.update_flags(dest);
            }

            RawOpCode::Ldi => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let addr = self.reg.get(Register::PC) + pc_offset;
                let miku_addr = self.mem.read(addr);

                self.reg.set(dest, self.mem.read(miku_addr));
                self.update_flags(dest);
            }

            RawOpCode::Lea => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let eff_addr = self.reg.get(Register::PC) + pc_offset;

                self.reg.set(dest, eff_addr);
                self.update_flags(dest);
            }

            RawOpCode::St => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let addr = self.reg.get(Register::PC) + pc_offset;

                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Sti => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let miku_addr = self.reg.get(Register::PC) + pc_offset;

                let addr = self.mem.read(miku_addr);
                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Str => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                let offset = sign_extend(raw_instr & 0x3F, 6);
                let addr = self.reg.get(base) + offset;

                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Trap => {
                let trap_code = TrapCode::from_u16(raw_instr & 0xFF);

                if let Some(trap_code) = trap_code {
                    match trap_code {
                        TrapCode::GetC => {
                            let mut buff = [0; 1];
                            io::stdin().read_exact(&mut buff).unwrap();

                            self.reg.set(Register::R0, buff[0] as u16);
                        }

                        TrapCode::Out => {
                            let ch = self.reg.get(Register::R0) as u8 as char;
                            print!("{}", ch);
                            io::stdout().flush().expect("Failed to flush stdout");
                        }

                        TrapCode::Puts => {
                            let mut miku_str = String::new();
                            let mut miku_addr = self.reg.get(Register::R0);
                            while self.mem.read(miku_addr) != 0x0000 {
                                let ch = self.mem.read(miku_addr) as u8 as char;
                                miku_str.push(ch);
                                miku_addr += 1;
                            }
                            print!("{miku_str}");
                            io::stdout().flush().expect("Failed to flush stdout");
                        }

                        TrapCode::In => {
                            print!("Enter a character : ");
                            io::stdout().flush().expect("Failed to flush stdout");
                            let ch = io::stdin()
                                .bytes()
                                .next()
                                .and_then(|result| result.ok())
                                .unwrap() as u16;
                            self.reg.set(Register::R0, ch);
                        }

                        TrapCode::PutsP => {
                            let mut miku_str = String::new();
                            let mut miku_addr = self.reg.get(Register::R0);

                            while self.mem.read(miku_addr) != 0x0000 {
                                let val = self.mem.read(miku_addr);
                                let c1 = (val & 0xFF) as u8 as char;
                                miku_str.push(c1);
                                let c2 = (val >> 8) as u8 as char;
                                if c2 != '\0' {
                                    miku_str.push(c2);
                                }
                                miku_addr += 1;
                            }
                            print!("{miku_str}");
                            io::stdout().flush().expect("Failed to flush stdout");
                        }

                        TrapCode::Halt => {
                            print!("Machine Halted");
                            io::stdout().flush().expect("Failed to flush stdout");
                            self.is_running = false;
                        }
                    }
                } else {
                    println!("Something fucked");
                    println!("{raw_instr}");
                }
            }
            RawOpCode::Rti => (),
            RawOpCode::Noop => (),
        };
    }

    fn update_flags(&mut self, register: Register) {
        let flag = CondFlag::from_reg_value(self.reg.get(register));
        self.reg.set(Register::COND, flag.to_u16().unwrap());
    }
}
