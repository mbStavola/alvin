use std::fmt;

use emulator::{Address, Constant, Register};

pub enum Opcode {
    Call(Address),
    Clear,
    Return,
    Goto(Address),
    CallFunction(Address),
    SkipEq(Register, Constant),
    SkipNEq(Register, Constant),
    SkipEqReg(Register, Register),
    Set(Register, Constant),
    AddAssign(Register, Constant),
    Copy(Register, Register),
    Or(Register, Register),
    And(Register, Register),
    Xor(Register, Register),
    AddAssignReg(Register, Register),
    SubAssignReg(Register, Register),
    ShiftRight(Register, Register),
    Subtract(Register, Register),
    ShiftLeft(Register, Register),
    SkipNEqReg(Register, Register),
    SetAddressReg(Address),
    JumpOffset(Constant),
    SetRand(Register, Constant),
    Draw(Register, Register, Constant),
    SkipKeyPress(Register),
    SkipNoKeyPress(Register),
    StoreDelayTimer(Register),
    StoreKeypress(Register),
    SetDelayTimer(Register),
    SetSoundTimer(Register),
    IncrementAddressReg(Register),
    StoreSpriteAddress(Register),
    BinaryCodedDecimal(Register),
    Dump(Register),
    Load(Register),
}

impl Opcode {
    pub fn from(first_byte: u8, second_byte: u8) -> Option<Opcode> {
        let nibbles = [
            (first_byte & 0xF0) >> 0x4,
            first_byte & 0xF,
            (second_byte & 0xF0) >> 0x4,
            second_byte & 0xF
        ];

        // println!("{:x}{:x}{:x}{:x}", nibbles[0], nibbles[1], nibbles[2], nibbles[3]);

        match nibbles[0] {
            0x0 => match (nibbles[2], nibbles[3]) {
                (0xE, 0x0) => Some(Opcode::Clear),
                (0xE, 0xE) => Some(Opcode::Return),
                _ => Some(Opcode::Call(build_address(nibbles)))
            },
            0x1 => Some(Opcode::Goto(build_address(nibbles))),
            0x2 => Some(Opcode::CallFunction(build_address(nibbles))),
            0x3 => Some(Opcode::SkipEq(nibbles[1], build_constant(nibbles))),
            0x4 => Some(Opcode::SkipNEq(nibbles[1], build_constant(nibbles))),
            0x5 => Some(Opcode::SkipEqReg(nibbles[1], nibbles[2])),
            0x6 => Some(Opcode::Set(nibbles[1], build_constant(nibbles))),
            0x7 => Some(Opcode::AddAssign(nibbles[1], build_constant(nibbles))),
            0x8 => match nibbles[3] {
                0x0 => Some(Opcode::Copy(nibbles[1], nibbles[2])),
                0x1 => Some(Opcode::Or(nibbles[1], nibbles[2])),
                0x2 => Some(Opcode::And(nibbles[1], nibbles[2])),
                0x3 => Some(Opcode::Xor(nibbles[1], nibbles[2])),
                0x4 => Some(Opcode::AddAssignReg(nibbles[1], nibbles[2])),
                0x5 => Some(Opcode::SubAssignReg(nibbles[1], nibbles[2])),
                0x6 => Some(Opcode::ShiftRight(nibbles[1], nibbles[2])),
                0x7 => Some(Opcode::Subtract(nibbles[1], nibbles[2])),
                0xE => Some(Opcode::ShiftLeft(nibbles[1], nibbles[2])),
                _ => None
            },
            0x9 => Some(Opcode::SkipNEqReg(nibbles[1], nibbles[2])),
            0xA => Some(Opcode::SetAddressReg(build_address(nibbles))),
            0xB => Some(Opcode::JumpOffset(build_constant(nibbles))),
            0xC => Some(Opcode::SetRand(nibbles[1], build_constant(nibbles))),
            0xD => Some(Opcode::Draw(nibbles[1], nibbles[2], nibbles[3])),
            0xE => match (nibbles[2], nibbles[3]) {
                (0x9, 0xE) => Some(Opcode::SkipKeyPress(nibbles[1])),
                (0xA, 0x1) => Some(Opcode::SkipNoKeyPress(nibbles[1])),
                _ => None
            },
            0xF => match (nibbles[2], nibbles[3]) {
                (0x0, 0x7) => Some(Opcode::StoreDelayTimer(nibbles[1])),
                (0x0, 0xA) => Some(Opcode::StoreKeypress(nibbles[1])),
                (0x1, 0x5) => Some(Opcode::SetDelayTimer(nibbles[1])),
                (0x1, 0x8) => Some(Opcode::SetSoundTimer(nibbles[1])),
                (0x1, 0xE) => Some(Opcode::IncrementAddressReg(nibbles[1])),
                (0x2, 0x9) => Some(Opcode::StoreSpriteAddress(nibbles[1])),
                (0x3, 0x3) => Some(Opcode::BinaryCodedDecimal(nibbles[1])),
                (0x5, 0x5) => Some(Opcode::Dump(nibbles[1])),
                (0x6, 0x5) => Some(Opcode::Load(nibbles[1])),
                _ => None
            }
            _ => None
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Opcode::Call(address) => {
                write!(f, "CALL\t{:x}", address)
            }
            Opcode::Clear => {
                write!(f, "CLEAR")
            }
            Opcode::Return => {
                write!(f, "RETURN")
            }
            Opcode::Goto(address) => {
                write!(f, "GOTO\t{:x}", address)
            }
            Opcode::CallFunction(address) => {
                write!(f, "CALLFUN\t{:x}", address)
            }
            Opcode::SkipEq(register, constant) => {
                write!(f, "SKIP_EQ\t{:x}\t{}", register, constant)
            }
            Opcode::SkipNEq(register, constant) => {
                write!(f, "SKIP_NEQ\t{:x}\t{}", register, constant)
            }
            Opcode::SkipEqReg(first, second) => {
                write!(f, "SKIP_EQ\t{:x}\t{:x}", first, second)
            }
            Opcode::Set(register, constant) => {
                write!(f, "SET\t{:x}\t{}", register, constant)
            }
            Opcode::AddAssign(register, constant) => {
                write!(f, "ADD_ASSIGN\t{:x}\t{}", register, constant)
            }
            Opcode::Copy(to, from) => {
                write!(f, "COPY\t{:x}\t{:x}", to, from)
            }
            Opcode::Or(first, second) => {
                write!(f, "OR\t{:x}\t{:x}", first, second)
            }
            Opcode::And(first, second) => {
                write!(f, "AND\t{:x}\t{:x}", first, second)
            }
            Opcode::Xor(first, second) => {
                write!(f, "XOR\t{:x}\t{:x}", first, second)
            }
            Opcode::AddAssignReg(first, second) => {
                write!(f, "ADD_ASSIGN\t{:x}\t{:x}", first, second)
            }
            Opcode::SubAssignReg(first, second) => {
                write!(f, "SUB_ASSIGN\t{:x}\t{:x}", first, second)
            }
            Opcode::ShiftRight(first, second) => {
                write!(f, "RSH\t{:x}\t{:x}", first, second)
            }
            Opcode::Subtract(first, second) => {
                write!(f, "SUB\t{:x}\t{:x}", first, second)
            }
            Opcode::ShiftLeft(first, second) => {
                write!(f, "LSH\t{:x}\t{:x}", first, second)
            }
            Opcode::SkipNEqReg(first, second) => {
                write!(f, "SKIP_NEQ\t{:x}\t{:x}", first, second)
            }
            Opcode::SetAddressReg(address) => {
                write!(f, "SET\tI\t{:x}", address)
            }
            Opcode::JumpOffset(constant) => {
                write!(f, "OFFSET\t{}", constant)
            }
            Opcode::SetRand(register, constant) => {
                write!(f, "RAND\t{:x}\t{}", register, constant)
            }
            Opcode::Draw(first, second, constant) => {
                write!(f, "DRAW\t{:x}\t{:x}\t{}", first, second, constant)
            }
            Opcode::SkipKeyPress(register) => {
                write!(f, "SKIP_KP\t{:x}", register)
            }
            Opcode::SkipNoKeyPress(register) => {
                write!(f, "SKIP_NKP\t{:x}", register)
            }
            Opcode::StoreDelayTimer(register) => {
                write!(f, "SET\t{:x}\tDELAY", register)
            }
            Opcode::StoreKeypress(register) => {
                write!(f, "SET\t{:x}\tKP", register)
            }
            Opcode::SetDelayTimer(register) => {
                write!(f, "SET\tDELAY\t{:x}", register)
            }
            Opcode::SetSoundTimer(register) => {
                write!(f, "SET\tSOUND\t{:x}", register)
            }
            Opcode::IncrementAddressReg(register) => {
                write!(f, "ADD\tI\t{:x}", register)
            }
            Opcode::StoreSpriteAddress(register) => {
                write!(f, "SET_SPRITE\t{:x}", register)
            }
            Opcode::BinaryCodedDecimal(register) => {
                write!(f, "BCD\t{:x}", register)
            }
            Opcode::Dump(register) => {
                write!(f, "DUMP\t{:x}", register)
            }
            Opcode::Load(register) => {
                write!(f, "LOAD\t{:x}", register)
            }
        }
    }
}

fn build_address(nibbles: [u8; 4]) -> Address {
    let first = nibbles[1] as u16;
    let third = nibbles[2] as u16;
    let second = nibbles[3] as u16;

    (first << 8) + (second << 4) + third
}

fn build_constant(nibbles: [u8; 4]) -> Constant {
    (nibbles[2] << 4) + nibbles[3]
}