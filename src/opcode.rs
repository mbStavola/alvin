use std::fmt;

use crate::emulator::{Address, Constant, Register};

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
    JumpOffset(Address),
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

#[derive(Debug)]
pub struct Data(pub Address);

impl Opcode {
    pub fn from(first_byte: u8, second_byte: u8) -> Result<Opcode, (Data, Data)> {
        let nibbles = (
            (first_byte & 0xF0) >> 0x4,
            first_byte & 0xF,
            (second_byte & 0xF0) >> 0x4,
            second_byte & 0xF
        );

        let opcode = match nibbles {
            (0x0, _, 0xE, 0x0) => Opcode::Clear,
            (0x0, _, 0xE, 0xE) => Opcode::Return,
            (0x0, _, _, _) => Opcode::Call(build_address(nibbles)),
            (0x1, _, _, _) => Opcode::Goto(build_address(nibbles)),
            (0x2, _, _, _) => Opcode::CallFunction(build_address(nibbles)),
            (0x3, _, _, _) => Opcode::SkipEq(nibbles.1, build_constant(nibbles)),
            (0x4, _, _, _) => Opcode::SkipNEq(nibbles.1, build_constant(nibbles)),
            (0x5, _, _, 0x0) => Opcode::SkipEqReg(nibbles.1, nibbles.2),
            (0x6, _, _, _) => Opcode::Set(nibbles.1, build_constant(nibbles)),
            (0x7, _, _, _) => Opcode::AddAssign(nibbles.1, build_constant(nibbles)),
            (0x8, _, _, 0x0) => Opcode::Copy(nibbles.1, nibbles.2),
            (0x8, _, _, 0x1) => Opcode::Or(nibbles.1, nibbles.2),
            (0x8, _, _, 0x2) => Opcode::And(nibbles.1, nibbles.2),
            (0x8, _, _, 0x3) => Opcode::Xor(nibbles.1, nibbles.2),
            (0x8, _, _, 0x4) => Opcode::AddAssignReg(nibbles.1, nibbles.2),
            (0x8, _, _, 0x5) => Opcode::SubAssignReg(nibbles.1, nibbles.2),
            (0x8, _, _, 0x6) => Opcode::ShiftRight(nibbles.1, nibbles.2),
            (0x8, _, _, 0x7) => Opcode::Subtract(nibbles.1, nibbles.2),
            (0x8, _, _, 0xE) => Opcode::ShiftLeft(nibbles.1, nibbles.2),
            (0x9, _, _, 0x0) => Opcode::SkipNEqReg(nibbles.1, nibbles.2),
            (0xA, _, _, _) => Opcode::SetAddressReg(build_address(nibbles)),
            (0xB, _, _, _) => Opcode::JumpOffset(build_address(nibbles)),
            (0xC, _, _, _) => Opcode::SetRand(nibbles.1, build_constant(nibbles)),
            (0xD, _, _, _) => Opcode::Draw(nibbles.1, nibbles.2, nibbles.3),
            (0xE, _, 0x9, 0xE) => Opcode::SkipKeyPress(nibbles.1),
            (0xE, _, 0xA, 0x1) => Opcode::SkipNoKeyPress(nibbles.1),
            (0xF, _, 0x0, 0x7) => Opcode::StoreDelayTimer(nibbles.1),
            (0xF, _, 0x0, 0xA) => Opcode::StoreKeypress(nibbles.1),
            (0xF, _, 0x1, 0x5) => Opcode::SetDelayTimer(nibbles.1),
            (0xF, _, 0x1, 0x8) => Opcode::SetSoundTimer(nibbles.1),
            (0xF, _, 0x1, 0xE) => Opcode::IncrementAddressReg(nibbles.1),
            (0xF, _, 0x2, 0x9) => Opcode::StoreSpriteAddress(nibbles.1),
            (0xF, _, 0x3, 0x3) => Opcode::BinaryCodedDecimal(nibbles.1),
            (0xF, _, 0x5, 0x5) => Opcode::Dump(nibbles.1),
            (0xF, _, 0x6, 0x5) => Opcode::Load(nibbles.1),
            _ => return Err(build_data(nibbles))
        };

        Ok(opcode)
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Opcode::Call(address) => {
                write!(f, "CALL\t{:#03x}", address)
            }
            Opcode::Clear => {
                write!(f, "CLS")
            }
            Opcode::Return => {
                write!(f, "RET")
            }
            Opcode::Goto(address) => {
                write!(f, "JMP\t{:#03x}", address)
            }
            Opcode::CallFunction(address) => {
                write!(f, "CALL\t{:#03x}", address)
            }
            Opcode::SkipEq(register, constant) => {
                write!(f, "SE\tV{:X}\t{}", register, constant)
            }
            Opcode::SkipNEq(register, constant) => {
                write!(f, "SNE\tV{:X}\t{}", register, constant)
            }
            Opcode::SkipEqReg(first, second) => {
                write!(f, "SE\tV{:X}\tV{:X}", first, second)
            }
            Opcode::Set(register, constant) => {
                write!(f, "SET\tV{:X}\t{}", register, constant)
            }
            Opcode::AddAssign(register, constant) => {
                write!(f, "ADDA\tV{:X}\t{}", register, constant)
            }
            Opcode::Copy(to, from) => {
                write!(f, "SET\tV{:X}\tV{:X}", to, from)
            }
            Opcode::Or(first, second) => {
                write!(f, "OR\tV{:X}\tV{:X}", first, second)
            }
            Opcode::And(first, second) => {
                write!(f, "AND\tV{:X}\tV{:X}", first, second)
            }
            Opcode::Xor(first, second) => {
                write!(f, "XOR\tV{:X}\tV{:X}", first, second)
            }
            Opcode::AddAssignReg(first, second) => {
                write!(f, "ADDA\tV{:X}\tV{:X}", first, second)
            }
            Opcode::SubAssignReg(first, second) => {
                write!(f, "SUBA\tV{:X}\tV{:X}", first, second)
            }
            Opcode::ShiftRight(first, second) => {
                write!(f, "SHR\tV{:X}\tV{:X}", first, second)
            }
            Opcode::Subtract(first, second) => {
                write!(f, "SUB\tV{:X}\tV{:X}", first, second)
            }
            Opcode::ShiftLeft(first, second) => {
                write!(f, "SHL\tV{:X}\tV{:X}", first, second)
            }
            Opcode::SkipNEqReg(first, second) => {
                write!(f, "SNE\tV{:X}\tV{:X}", first, second)
            }
            Opcode::SetAddressReg(address) => {
                write!(f, "SET\tI\t{:#03x}", address)
            }
            Opcode::JumpOffset(address) => {
                write!(f, "OFF\t{:#03x}", address)
            }
            Opcode::SetRand(register, constant) => {
                write!(f, "RND\tV{:X}\t{}", register, constant)
            }
            Opcode::Draw(first, second, constant) => {
                write!(f, "DRW\tV{:X}\tV{:X}\t{}", first, second, constant)
            }
            Opcode::SkipKeyPress(register) => {
                write!(f, "SKP\tV{:X}", register)
            }
            Opcode::SkipNoKeyPress(register) => {
                write!(f, "SNKP\tV{:X}", register)
            }
            Opcode::StoreDelayTimer(register) => {
                write!(f, "SET\tV{:X}\tDELAY", register)
            }
            Opcode::StoreKeypress(register) => {
                write!(f, "SET\tV{:X}\tKEY", register)
            }
            Opcode::SetDelayTimer(register) => {
                write!(f, "SET\tDELAY\tV{:X}", register)
            }
            Opcode::SetSoundTimer(register) => {
                write!(f, "SET\tSOUND\tV{:X}", register)
            }
            Opcode::IncrementAddressReg(register) => {
                write!(f, "ADD\tI\tV{:X}", register)
            }
            Opcode::StoreSpriteAddress(register) => {
                write!(f, "SPRT\tV{:X}", register)
            }
            Opcode::BinaryCodedDecimal(register) => {
                write!(f, "BCD\tV{:X}", register)
            }
            Opcode::Dump(register) => {
                write!(f, "DUMP\tV{:X}", register)
            }
            Opcode::Load(register) => {
                write!(f, "LOAD\tV{:X}", register)
            }
        }
    }
}

fn build_data(nibbles: (u8, u8, u8, u8)) -> (Data, Data) {
    let first = nibbles.0 as u16;
    let second = nibbles.1 as u16;

    let third = nibbles.2 as u16;
    let fourth = nibbles.3 as u16;

    let left = (first << 4) | second;
    let right = (third << 4) | fourth;

    (Data(left), Data(right))
}

fn build_address(nibbles: (u8, u8, u8, u8)) -> Address {
    let first = nibbles.1 as u16;
    let second = nibbles.2 as u16;
    let third = nibbles.3 as u16;

    (first << 8) | (second << 4) | third
}

fn build_constant(nibbles: (u8, u8, u8, u8)) -> Constant {
    (nibbles.2 << 4) | nibbles.3
}