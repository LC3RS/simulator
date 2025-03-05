use num_traits::{FromPrimitive, ToPrimitive};

use crate::{
    enums::{CondFlag, RawOpCode, Register},
    memory::{MemoryManager, RegisterManager},
    utils::sign_extend,
};

#[derive(Default)]
pub struct Machine {
    reg: RegisterManager,
    mem: MemoryManager,
    is_running: bool,
}

impl Machine {
    pub fn print_registers(&self) {
        self.reg.debug_all();
    }

    pub fn run(&mut self) {
        self.is_running = true;

        while self.is_running {
            let raw_instr = self.fetch();
            self.decode_and_execute(raw_instr);
        }
    }

    fn fetch(&mut self) -> u16 {
        let instr = self.mem.read(self.reg.get(Register::PC));
        self.reg.incr(Register::PC);
        instr
    }

    fn decode_and_execute(&mut self, raw_instr: u16) {
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

                self.mem.write(self.mem.read(miku_addr), self.reg.get(src));
            }

            RawOpCode::Str => {
                let src = Register::from_u16((raw_instr >> 9) & 0x7).unwrap();
                let base = Register::from_u16((raw_instr >> 6) & 0x7).unwrap();
                let offset = sign_extend(raw_instr & 0x3F, 6);
                let addr = self.reg.get(base) + offset;

                self.mem.write(addr, self.reg.get(src));
            }

            RawOpCode::Noop => (),
            _ => (), // TODO: remove after complete
        };
    }

    fn update_flags(&mut self, register: Register) {
        let flag = CondFlag::from_reg_value(self.reg.get(register));
        self.reg.set(Register::COND, flag.to_u16().unwrap());
    }
}
