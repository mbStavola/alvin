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

pub struct Data(pub Address);

impl Opcode {
    pub fn from(first_byte: u8, second_byte: u8) -> Result<Opcode, (Data, Data)> {
        let nibbles = [
            (first_byte & 0xF0) >> 0x4,
            first_byte & 0xF,
            (second_byte & 0xF0) >> 0x4,
            second_byte & 0xF
        ];

        // println!("{:x}{:x}{:x}{:x}", nibbles[0], nibbles[1], nibbles[2], nibbles[3]);

        match nibbles[0] {
            0x0 => match (nibbles[2], nibbles[3]) {
                (0xE, 0x0) => Ok(Opcode::Clear),
                (0xE, 0xE) => Ok(Opcode::Return),
                _ => Ok(Opcode::Call(build_address(nibbles)))
            },
            0x1 => Ok(Opcode::Goto(build_address(nibbles))),
            0x2 => Ok(Opcode::CallFunction(build_address(nibbles))),
            0x3 => Ok(Opcode::SkipEq(nibbles[1], build_constant(nibbles))),
            0x4 => Ok(Opcode::SkipNEq(nibbles[1], build_constant(nibbles))),
            0x5 => Ok(Opcode::SkipEqReg(nibbles[1], nibbles[2])),
            0x6 => Ok(Opcode::Set(nibbles[1], build_constant(nibbles))),
            0x7 => Ok(Opcode::AddAssign(nibbles[1], build_constant(nibbles))),
            0x8 => match nibbles[3] {
                0x0 => Ok(Opcode::Copy(nibbles[1], nibbles[2])),
                0x1 => Ok(Opcode::Or(nibbles[1], nibbles[2])),
                0x2 => Ok(Opcode::And(nibbles[1], nibbles[2])),
                0x3 => Ok(Opcode::Xor(nibbles[1], nibbles[2])),
                0x4 => Ok(Opcode::AddAssignReg(nibbles[1], nibbles[2])),
                0x5 => Ok(Opcode::SubAssignReg(nibbles[1], nibbles[2])),
                0x6 => Ok(Opcode::ShiftRight(nibbles[1], nibbles[2])),
                0x7 => Ok(Opcode::Subtract(nibbles[1], nibbles[2])),
                0xE => Ok(Opcode::ShiftLeft(nibbles[1], nibbles[2])),
                _ => Err(build_data(nibbles))
            },
            0x9 => Ok(Opcode::SkipNEqReg(nibbles[1], nibbles[2])),
            0xA => Ok(Opcode::SetAddressReg(build_address(nibbles))),
            0xB => Ok(Opcode::JumpOffset(build_constant(nibbles))),
            0xC => Ok(Opcode::SetRand(nibbles[1], build_constant(nibbles))),
            0xD => Ok(Opcode::Draw(nibbles[1], nibbles[2], nibbles[3])),
            0xE => match (nibbles[2], nibbles[3]) {
                (0x9, 0xE) => Ok(Opcode::SkipKeyPress(nibbles[1])),
                (0xA, 0x1) => Ok(Opcode::SkipNoKeyPress(nibbles[1])),
                _ => Err(build_data(nibbles))
            },
            0xF => match (nibbles[2], nibbles[3]) {
                (0x0, 0x7) => Ok(Opcode::StoreDelayTimer(nibbles[1])),
                (0x0, 0xA) => Ok(Opcode::StoreKeypress(nibbles[1])),
                (0x1, 0x5) => Ok(Opcode::SetDelayTimer(nibbles[1])),
                (0x1, 0x8) => Ok(Opcode::SetSoundTimer(nibbles[1])),
                (0x1, 0xE) => Ok(Opcode::IncrementAddressReg(nibbles[1])),
                (0x2, 0x9) => Ok(Opcode::StoreSpriteAddress(nibbles[1])),
                (0x3, 0x3) => Ok(Opcode::BinaryCodedDecimal(nibbles[1])),
                (0x5, 0x5) => Ok(Opcode::Dump(nibbles[1])),
                (0x6, 0x5) => Ok(Opcode::Load(nibbles[1])),
                _ => Err(build_data(nibbles))
            }
            _ => Err(build_data(nibbles))
        }
    }
}

impl fmt::Debug for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Opcode::Call(address) => {
                write!(f, "CALL\t{:#04x}", address)
            }
            Opcode::Clear => {
                write!(f, "CLS")
            }
            Opcode::Return => {
                write!(f, "RET")
            }
            Opcode::Goto(address) => {
                write!(f, "JMP\t{:#04x}", address)
            }
            Opcode::CallFunction(address) => {
                write!(f, "CALL\t{:#04x}", address)
            }
            Opcode::SkipEq(register, constant) => {
                write!(f, "SE\t{:#03x}\t{}", register, constant)
            }
            Opcode::SkipNEq(register, constant) => {
                write!(f, "SNE\t{:#03x}\t{}", register, constant)
            }
            Opcode::SkipEqReg(first, second) => {
                write!(f, "SE\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::Set(register, constant) => {
                write!(f, "SET\t{:#03x}\t{}", register, constant)
            }
            Opcode::AddAssign(register, constant) => {
                write!(f, "ADDA\t{:#03x}\t{}", register, constant)
            }
            Opcode::Copy(to, from) => {
                write!(f, "SET\t{:#03x}\t{:#03x}", to, from)
            }
            Opcode::Or(first, second) => {
                write!(f, "OR\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::And(first, second) => {
                write!(f, "AND\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::Xor(first, second) => {
                write!(f, "XOR\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::AddAssignReg(first, second) => {
                write!(f, "ADDA\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::SubAssignReg(first, second) => {
                write!(f, "SUBA\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::ShiftRight(first, second) => {
                write!(f, "SHR\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::Subtract(first, second) => {
                write!(f, "SUB\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::ShiftLeft(first, second) => {
                write!(f, "SHL\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::SkipNEqReg(first, second) => {
                write!(f, "SNE\t{:#03x}\t{:#03x}", first, second)
            }
            Opcode::SetAddressReg(address) => {
                write!(f, "SET\tI\t{:#04x}", address)
            }
            Opcode::JumpOffset(constant) => {
                write!(f, "OFF\t{}", constant)
            }
            Opcode::SetRand(register, constant) => {
                write!(f, "RND\t{:#03x}\t{}", register, constant)
            }
            Opcode::Draw(first, second, constant) => {
                write!(f, "DRW\t{:#03x}\t{:#03x}\t{}", first, second, constant)
            }
            Opcode::SkipKeyPress(register) => {
                write!(f, "SKP\t{:#03x}", register)
            }
            Opcode::SkipNoKeyPress(register) => {
                write!(f, "SNKP\t{:#03x}", register)
            }
            Opcode::StoreDelayTimer(register) => {
                write!(f, "SET\t{:#03x}\tDELAY", register)
            }
            Opcode::StoreKeypress(register) => {
                write!(f, "SET\t{:#03x}\tKEY", register)
            }
            Opcode::SetDelayTimer(register) => {
                write!(f, "SET\tDELAY\t{:#03x}", register)
            }
            Opcode::SetSoundTimer(register) => {
                write!(f, "SET\tSOUND\t{:#03x}", register)
            }
            Opcode::IncrementAddressReg(register) => {
                write!(f, "ADD\tI\t{:#03x}", register)
            }
            Opcode::StoreSpriteAddress(register) => {
                write!(f, "SPRT\t{:#03x}", register)
            }
            Opcode::BinaryCodedDecimal(register) => {
                write!(f, "BCD\t{:#03x}", register)
            }
            Opcode::Dump(register) => {
                write!(f, "DUMP\t{:#03x}", register)
            }
            Opcode::Load(register) => {
                write!(f, "LOAD\t{:#03x}", register)
            }
        }
    }
}

fn build_data(nibbles: [u8; 4]) -> (Data, Data) {
    let first = nibbles[0] as u16;
    let second = nibbles[1] as u16;

    let third = nibbles[2] as u16;
    let fourth = nibbles[3] as u16;

    let left = (first << 4) + second;
    let right = (third << 4) + fourth;

    (Data(left), Data(right))
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