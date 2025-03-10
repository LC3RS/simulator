use byteorder::{BigEndian, ReadBytesExt};
use num_traits::{FromPrimitive, ToPrimitive, WrappingAdd};
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
                    addr = addr.wrapping_add(1);
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
                    self.reg.set(dest, self.reg.get(src1).wrapping_add(imm5));
                } else {
                    let src2 = Register::from_u16(raw_instr & 0x7).unwrap();
                    self.reg
                        .set(dest, self.reg.get(src1).wrapping_add(self.reg.get(src2)));
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
                let addr = self.reg.get(Register::PC).wrapping_add(pc_offset);

                self.reg.set(dest, self.mem.read(addr));
                self.update_flags(dest);
            }

            RawOpCode::Ldr => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                let offset = sign_extend(raw_instr & 0x3F, 6);
                let data = self.mem.read(self.reg.get(base).wrapping_add(offset));

                self.reg.set(dest, data);
                self.update_flags(dest);
            }

            RawOpCode::Ldi => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let addr = self.reg.get(Register::PC).wrapping_add(pc_offset);
                let miku_addr = self.mem.read(addr);

                self.reg.set(dest, self.mem.read(miku_addr));
                self.update_flags(dest);
            }

            RawOpCode::Lea => {
                let dest = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let eff_addr = self.reg.get(Register::PC).wrapping_add(pc_offset);

                self.reg.set(dest, eff_addr);
                self.update_flags(dest);
            }

            RawOpCode::St => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let addr = self.reg.get(Register::PC).wrapping_add(pc_offset);

                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Sti => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);
                let miku_addr = self.reg.get(Register::PC).wrapping_add(pc_offset);

                let addr = self.mem.read(miku_addr);
                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Str => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                let offset = sign_extend(raw_instr & 0x3F, 6);
                let addr = self.reg.get(base).wrapping_add(offset);

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
                                miku_addr = miku_addr.wrapping_add(1);
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
                                miku_addr = miku_addr.wrapping_add(1);
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::enums::Register::COND;
    #[test]
    fn test_add() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::R0, 56);
        test_mach.reg.set(Register::R1, 0);
        test_mach.reg.set(Register::R2, 4);
        test_mach.reg.set(Register::R4, 7);
        test_mach.reg.set(Register::R7, 13);

        test_mach.decode_and_execute(0b0001_011_000_0_00_001);
        assert_eq!(test_mach.reg.get(Register::R3), 56);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0001_011_000_0_00_111);
        assert_eq!(test_mach.reg.get(Register::R3), 69);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0001_100_010_1_10001);
        assert_eq!(test_mach.reg.get(Register::R4), 0b1111_1111_1111_0101);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
        test_mach.decode_and_execute(0b0001_111_111_1_10011);
        assert_eq!(test_mach.reg.get(Register::R7), 0);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Zero as u16);
    }

    #[test]
    fn test_and() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::R0, 0b0010_1010_1110_1000);
        test_mach.reg.set(Register::R1, 0b1010_1010_1010_1010);
        test_mach.reg.set(Register::R2, 0b0000_0000_0000_0000);
        test_mach.reg.set(Register::R4, 0b1111_1111_1111_1111);
        test_mach.reg.set(Register::R7, 0b0101_1100_0100_1110);

        test_mach.decode_and_execute(0b0101_011_000_0_00_010);
        assert_eq!(test_mach.reg.get(Register::R3), 0b0000_0000_0000_0000);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Zero as u16);
        test_mach.decode_and_execute(0b0101_011_000_0_00_111);
        assert_eq!(test_mach.reg.get(Register::R3), 0b0000_1000_0100_1000);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0101_010_100_1_00110);
        assert_eq!(test_mach.reg.get(Register::R2), 0b0000_0000_0000_0110);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0101_111_100_1_10011);
        assert_eq!(test_mach.reg.get(Register::R7), 0b1111_1111_1111_0011);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
    }

    #[test]
    fn test_not() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::R0, 0b0010_1010_1110_1000);
        test_mach.reg.set(Register::R1, 0b1010_1010_1010_1010);
        test_mach.reg.set(Register::R2, 0b1111_1111_1111_1111);

        test_mach.decode_and_execute(0b1001_011_000_111111);
        assert_eq!(test_mach.reg.get(Register::R3), 0b1101_0101_0001_0111);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
        test_mach.decode_and_execute(0b1001_011_001_111111);
        assert_eq!(test_mach.reg.get(Register::R3), 0b0101_0101_0101_0101);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b1001_110_010_111111);
        assert_eq!(test_mach.reg.get(Register::R6), 0b0000_0000_0000_0000);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Zero as u16);
    }

    #[test]
    fn test_br() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0010_1010_1110_1000);
        test_mach.reg.set(Register::COND, 0b010);

        test_mach.decode_and_execute(0b0000_1_0_0_000100110);
        assert_eq!(test_mach.reg.get(Register::PC), 0b0010_1010_1110_1000);
        test_mach.decode_and_execute(0b0000_0_1_0_000100110);
        assert_eq!(test_mach.reg.get(Register::PC), 0b0010_1011_0000_1110);
    }

    #[test]
    fn test_jmp() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0010_1010_1110_1000);
        test_mach.reg.set(Register::R0, 15);
        test_mach.reg.set(Register::R5, 69);

        test_mach.decode_and_execute(0b1100_000_101_000000);
        assert_eq!(test_mach.reg.get(Register::PC), 69);
        test_mach.decode_and_execute(0b1100_000_000_000000);
        assert_eq!(test_mach.reg.get(Register::PC), 15);
    }

    #[test]
    fn test_jsr() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0010_1010_1110_1000);
        test_mach.reg.set(Register::R5, 420);

        test_mach.decode_and_execute(0b0100_1_01001010110);
        assert_eq!(test_mach.reg.get(Register::PC), 0b0010_1101_0011_1110);
        test_mach.decode_and_execute(0b0100_0_00_101_000000);
        assert_eq!(test_mach.reg.get(Register::PC), 420);
    }

    #[test]
    fn test_ld() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0010_1010_1110_1000);
        test_mach.mem.write(0b0010_1011_0011_1110, 1205);
        test_mach.mem.write(0b0010_1010_1111_1100, 65142);

        test_mach.decode_and_execute(0b0010_101_001010110);
        assert_eq!(test_mach.reg.get(Register::R5), 1205);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0010_001_000010100);
        assert_eq!(test_mach.reg.get(Register::R1), 65142);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
    }

    #[test]
    fn test_ldi() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0010_1010_1110_1000);
        test_mach
            .mem
            .write(0b0010_1011_0011_1110, 0b0010_1010_1111_1100);
        test_mach
            .mem
            .write(0b0010_1010_1111_1100, 0b1110_0011_0111_0101);
        test_mach.mem.write(0b1110_0011_0111_0101, 0);

        test_mach.decode_and_execute(0b1010_101_001010110);
        assert_eq!(test_mach.reg.get(Register::R5), 0b1110_0011_0111_0101);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
        test_mach.decode_and_execute(0b1010_001_000010100);
        assert_eq!(test_mach.reg.get(Register::R1), 0);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Zero as u16);
    }

    #[test]
    fn test_ldr() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::R0, 0b0010_1010_0001_1110);
        test_mach.reg.set(Register::R4, 0b0011_1100_1111_0110);
        test_mach.mem.write(0b0010_1010_0000_0011, 5087);
        test_mach.mem.write(0b0011_1101_0000_1100, 63251);

        test_mach.decode_and_execute(0b0110_101_000_100101);
        assert_eq!(test_mach.reg.get(Register::R5), 5087);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b0110_100_100_010110);
        assert_eq!(test_mach.reg.get(Register::R4), 63251);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Neg as u16);
    }

    #[test]
    fn test_lea() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b0111_0101_1011_0110);

        test_mach.decode_and_execute(0b1110_101_001111101);
        assert_eq!(test_mach.reg.get(Register::R5), 0b0111_0110_0011_0011);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
        test_mach.decode_and_execute(0b1110_100_111110001);
        assert_eq!(test_mach.reg.get(Register::R4), 0b0111_0101_1010_0111);
        assert_eq!(test_mach.reg.get(Register::COND), CondFlag::Pos as u16);
    }

    #[test]
    fn test_st() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b1001_1001_0111_1001);
        test_mach.reg.set(Register::R6, 1131);
        test_mach.reg.set(Register::R2, 9999);

        test_mach.decode_and_execute(0b0011_110_000101111);
        assert_eq!(test_mach.mem.read(0b1001_1001_1010_1000), 1131);
        test_mach.decode_and_execute(0b0011_010_100001011);
        assert_eq!(test_mach.mem.read(0b1001_1000_1000_0100), 9999);
    }

    #[test]
    fn test_sti() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::PC, 0b1001_1011_1001_1010);
        test_mach
            .mem
            .write(0b1001_1011_1100_1001, 0b1000_0011_1011_1111);
        test_mach
            .mem
            .write(0b1001_1010_1010_0101, 0b0111_1001_1000_1101);
        test_mach.reg.set(Register::R6, 6969);
        test_mach.reg.set(Register::R2, 1034);

        test_mach.decode_and_execute(0b1011_110_000101111);
        assert_eq!(test_mach.mem.read(0b1000_0011_1011_1111), 6969);
        test_mach.decode_and_execute(0b1011_010_100001011);
        assert_eq!(test_mach.mem.read(0b0111_1001_1000_1101), 1034);
    }

    #[test]
    fn test_str() {
        let mut test_mach = Machine::default();
        test_mach.reg.set(Register::R0, 0b1001_0100_1010_0001);
        test_mach.reg.set(Register::R4, 0b0111_1000_0110_1000);
        test_mach.reg.set(Register::R6, 38292);
        test_mach.reg.set(Register::R2, 15503);

        test_mach.decode_and_execute(0b0111_110_000_101111);
        assert_eq!(test_mach.mem.read(0b1001_0100_1001_0000), 38292);
        test_mach.decode_and_execute(0b0111_010_100_001011);
        assert_eq!(test_mach.mem.read(0b0111_1000_0111_0011), 15503);
    }

    #[test]
    fn test_trap() {
        //idk how to test this shit
        let mut test_mach = Machine::default();
        test_mach.decode_and_execute(0b1111_0000_00100000);
        test_mach.decode_and_execute(0b1111_0000_00100001);
        test_mach.decode_and_execute(0b1111_0000_00100010);
        test_mach.decode_and_execute(0b1111_0000_00100011);
        test_mach.decode_and_execute(0b1111_0000_00100100);
        test_mach.decode_and_execute(0b1111_0000_00100101);
    }
}
