use num_traits::ToPrimitive;

use crate::{
    constants::{MAX_MEMORY, PC_START},
    enums::Register,
};

pub struct RegisterManager {
    registers: [u16; 11],
}

impl Default for RegisterManager {
    fn default() -> Self {
        Self {
            registers: [0, 0, 0, 0, 0, 0, 0, 0, PC_START, 0, 0],
        }
    }
}

impl RegisterManager {
    pub fn get(&self, reg: Register) -> u16 {
        self.registers[reg.to_usize().unwrap()]
    }

    pub fn set(&mut self, reg: Register, val: u16) {
        self.registers[reg.to_usize().unwrap()] = val;
    }

    pub fn incr(&mut self, reg: Register) {
        self.registers[reg.to_usize().unwrap()] += 1;
    }

    pub fn incr_by(&mut self, reg: Register, val: u16) {
        self.registers[reg.to_usize().unwrap()] += val;
    }

    pub fn copy(&mut self, sink: Register, src: Register) {
        self.registers[sink.to_usize().unwrap()] = self.registers[src.to_usize().unwrap()];
    }

    pub fn debug_all(&self) {
        for reg in &self.registers {
            print!("{reg} ");
        }
        println!();
    }
}

pub struct MemoryManager {
    memory: [u16; MAX_MEMORY],
}

impl Default for MemoryManager {
    fn default() -> Self {
        Self {
            memory: [0; MAX_MEMORY],
        }
    }
}

impl MemoryManager {
    pub fn read(&self, addr: u16) -> u16 {
        // TODO: implement memory mapped addresses
        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }
}
