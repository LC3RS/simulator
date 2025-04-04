use std::io::{self, Read};

use num_traits::ToPrimitive;

use crate::{
    constants::{MAX_MEMORY, PC_START},
    enums::{MemMappedReg, Register},
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
        self.registers[reg.to_usize().unwrap()] =
            self.registers[reg.to_usize().unwrap()].wrapping_add(1);
    }

    pub fn incr_by(&mut self, reg: Register, val: u16) {
        self.registers[reg.to_usize().unwrap()] =
            self.registers[reg.to_usize().unwrap()].wrapping_add(val);
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
    pub fn read(&mut self, addr: u16) -> u16 {
        if addr == MemMappedReg::Kbsr.to_u16().unwrap() {
            let mut buf = [0; 1];
            io::stdin().read_exact(&mut buf).unwrap();
            if buf[0] != 0 {
                self.write(MemMappedReg::Kbsr.to_u16().unwrap(), 1 << 15);
                self.write(MemMappedReg::Kbdr.to_u16().unwrap(), buf[0] as u16);
            } else {
                self.write(MemMappedReg::Kbsr.to_u16().unwrap(), 0);
            }
        }

        self.memory[addr as usize]
    }

    pub fn write(&mut self, addr: u16, val: u16) {
        self.memory[addr as usize] = val;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_api() {
        let mut reg = RegisterManager::default();

        assert_eq!(reg.get(Register::PC), 0x3000);

        reg.set(Register::R0, 0x69);
        assert_eq!(reg.get(Register::R0), 0x69);

        reg.copy(Register::R7, Register::R0);
        assert_eq!(reg.get(Register::R7), 0x69);

        reg.incr(Register::R0);
        assert_eq!(reg.get(Register::R0), 0x6a);

        reg.incr_by(Register::R0, 5);
        assert_eq!(reg.get(Register::R0), 0x6f);

        reg.debug_all();
    }

    #[test]
    fn test_memory_api() {
        let mut mem = MemoryManager::default();

        mem.write(0, 0x69);
        assert_eq!(mem.read(0), 0x69);

        mem.write(0xffff, 0x7f);
        assert_eq!(mem.read(0xffff), 0x7f);
    }
}
