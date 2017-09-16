use emulator::{Address, Constant, Register};

#[derive(Debug)]
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

fn build_address(nibbles: [u8; 4]) -> Address {
    let first = nibbles[1] as u16;
    let third = nibbles[2] as u16;
    let second = nibbles[3] as u16;

    (first << 8) + (second << 4) + third
}

fn build_constant(nibbles: [u8; 4]) -> Constant {
    (nibbles[2] << 4) + nibbles[3]
}