use num_traits::{FromPrimitive, ToPrimitive};

use crate::{
    constants::{MAX_MEMORY, PC_START},
    enums::{CondFlag, DecodedInstr, RawOpCode},
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
            let decoded_instr = self.decode(raw_instr);
            self.execute(decoded_instr)
        }
    }

    fn fetch(&mut self) -> u16 {
        let instr = self.memory[self.pc as usize];
        self.pc += 1;
        instr
    }

    fn decode(&self, raw_instr: u16) -> DecodedInstr {
        let raw_op = RawOpCode::from_u16(raw_instr >> 12).unwrap();

        match raw_op {
            //RawOpCode::Add => {}
            RawOpCode::Noop => DecodedInstr::Noop,
            _ => DecodedInstr::Noop, // TODO: remove after complete
        }
    }

    fn execute(&self, decoded_instr: DecodedInstr) {
        // TODO: implement execute
        match decoded_instr {
            _ => (),
        }
    }

    fn update_flags(&mut self, reg_idx: usize) {
        self.cond = CondFlag::from_reg_value(self.registers[reg_idx])
            .to_u16()
            .unwrap();
    }
}
