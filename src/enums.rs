use num_derive::{FromPrimitive, ToPrimitive};

#[repr(usize)]
#[derive(FromPrimitive, ToPrimitive, Clone, Copy)]
pub enum Register {
    R0 = 0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    PC,
    COND,
    COUNT,
}

#[repr(u8)]
#[derive(FromPrimitive, ToPrimitive)]
// Raw opcode values
pub enum RawOpCode {
    Br = 0,
    Add,
    Ld,
    St,
    Jsr,
    And,
    Ldr,
    Str,
    Not,
    Ldi,
    Sti,
    Jmp, // JMP/RET
    Noop,
    Lea,
    Trap, // HALT
}

#[repr(u16)]
#[derive(ToPrimitive, FromPrimitive)]
//Condition Flags
pub enum CondFlag {
    Pos = 1 << 0,
    Zero = 1 << 1,
    Neg = 1 << 2,
}

#[repr(u8)]
#[derive(ToPrimitive, FromPrimitive)]
pub enum TrapCode {
    GetC = 0x20,
    Out,
    Puts,
    In,
    PutsP,
    Halt,
}

impl CondFlag {
    pub fn from_reg_value(val: u16) -> Self {
        if val == 0 {
            Self::Zero
        } else if (val >> 15) != 0 {
            Self::Neg
        } else {
            Self::Pos
        }
    }
}
