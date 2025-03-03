use num_derive::{FromPrimitive, ToPrimitive};

pub enum Register {
    R0,
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

// Instructions decoded from the opcodes
pub enum DecodedInstr {
    Add {
        dest: Register,
        src1: Register,
        src2: Register,
    },

    AddImm {
        dest: Register,
        src: Register,
        imm: i16,
    },

    And {
        dest: Register,
        src1: Register,
        src2: Register,
    },

    AndImm {
        dest: Register,
        src: Register,
        imm: i16,
    },

    Br {
        n: bool,
        z: bool,
        p: bool,
        offset: i16, // FIX: offset9
    },

    Jmp {
        base: Register,
    },

    Jsr {
        offset: i16, // FIX: offset11
    },

    Jsrr {
        base: Register,
    },

    Ld {
        dest: Register,
        offset: i16, // FIX: offset9
    },

    Ldi {
        dest: Register,
        offset: i16, // FIX: offset9
    },

    Ldr {
        dest: Register,
        base: Register,
        offset: i16, //FIX: offset6
    },

    Lea {
        dest: Register,
        offset: i16, //FIX: offset9
    },

    Not {
        dest: Register,
        src: Register,
    },

    Ret,

    St {
        src: Register,
        offset: i16, // FIX: offset9
    },

    Sti {
        src: Register,
        offset: i16, // FIX: offset9
    },

    Str {
        src: Register,
        base: Register,
        offset: i16, // FIX: offset6
    },

    Trap {
        trapvect: u8,
    },

    Noop,
}

#[repr(u16)]
#[derive(ToPrimitive, FromPrimitive)]
//Condition Flags
pub enum CondFlag {
    Pos = 1 << 0,
    Zero = 1 << 1,
    Neg = 1 << 2,
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
