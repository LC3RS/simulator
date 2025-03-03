use num_traits::{sign, FromPrimitive, ToPrimitive};

use crate::{
    constants::{MAX_MEMORY, PC_START},
    enums::{CondFlag, RawOpCode, Register},
    utils::sign_extend,
};

pub struct Machine {
    registers: [u16; 8],
    pc: u16,
    cond: u16,
    memory: [u16; MAX_MEMORY],
    is_running: bool,
}

impl Default for Machine {
    fn default() -> Self {
        Machine {
            registers: [0; 8],
            pc: PC_START,
            cond: 0,
            memory: [0; MAX_MEMORY],
            is_running: false,
        }
    }
}

impl Machine {
    pub fn print_registers(&self) {
        for reg in &self.registers {
            print!("{reg} ");
        }
        println!();
    }

    pub fn run(&mut self) {
        self.is_running = true;

        while self.is_running {
            let raw_instr = self.fetch();
            self.decode_and_execute(raw_instr);
        }
    }

    fn mem_read(&self, address: u16){
        self.memory[address as usize]
    }

    fn fetch(&mut self) -> u16 {
        let instr = self.memory[self.pc as usize];
        self.pc += 1;
        instr
    }

    fn decode_and_execute(&mut self, raw_instr: u16) {
        let raw_op = RawOpCode::from_u16(raw_instr >> 12).unwrap();

        match raw_op {
            RawOpCode::Add => {
                let dest = (raw_instr >> 9) & 0x7;
                let src1 = (raw_instr >> 6) & 0x7;

                // Check if we are in immediate mode
                let imm_flag = (raw_instr >> 5) & 0x1;

                if imm_flag == 1 {
                    let imm5 = sign_extend(raw_instr & 0x1F, 5);
                    self.registers[dest as usize] = self.registers[src1 as usize] + imm5;
                } else {
                    let src2 = raw_instr & 0x7;
                    self.registers[dest as usize] =
                        self.registers[src1 as usize] + self.registers[src2 as usize];
                }

                self.update_flags(dest as usize);
            }

            RawOpCode::And => {
                let dest = (raw_instr >> 9) & 0x7;
                let src1 = (raw_instr >> 6) & 0x7;

                // Check if we are in immediate mode
                let imm_flag = (raw_instr >> 5) & 0x1;

                if imm_flag == 1 {
                    let imm5 = sign_extend(raw_instr & 0x1F, 5);
                    self.registers[dest as usize] = self.registers[src1 as usize] & imm5;
                } else {
                    let src2 = raw_instr & 0x7;
                    self.registers[dest as usize] =
                        self.registers[src1 as usize] & self.registers[src2 as usize];
                }

                self.update_flags(dest as usize);
            }

            RawOpCode::Not => {
                let dest = (raw_instr >> 9) & 0x7;
                let src = (raw_instr >> 6) & 0x7;

                self.registers[dest as usize] = !self.registers[src as usize];

                self.update_flags(dest as usize);
            }

            RawOpCode::Br => {
                let cond_flag = (raw_instr >> 9) & 0x7;
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);

                if (cond_flag & self.cond)!=0 {
                    self.pc += pc_offset;
                }
            }

            RawOpCode::Jmp => {
                let base = (raw_instr >> 6) & 0x7;

                self.pc = self.registers[base as usize];
            }

            RawOpCode::Jsr => {
                let miku_bit = (raw_instr >> 11) & 0x1;
                self.registers[7] = self.pc;

                if miku_bit==1 {
                    let pc_offset = sign_extend(raw_instr & 0x7FF, 11);
                    self.pc += pc_offset;
                }
                else{
                    let base = (raw_instr >>6) & 0x7;
                    self.pc = self.registers[base as usize];
                }
            }

            RawOpCode::Ld => {
                let dest = (raw_instr >> 9) & 0x7;
                let pc_offset = sign_extend(raw_instr & 0x1FF, 9);

                self.registers[dest as usize] = mem_read(self.pc + pc_offset);
                self.update_flags(dest as usize);
            }


            RawOpCode::Noop => (),
            _ => (), // TODO: remove after complete
        };
    }

    fn update_flags(&mut self, reg_idx: usize) {
        self.cond = CondFlag::from_reg_value(self.registers[reg_idx])
            .to_u16()
            .unwrap();
    }
}
