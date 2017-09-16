use std::time::Instant;
use std::collections::VecDeque;

use opcode::Opcode;

pub type Address = u16;
pub type Constant = u8;
pub type Register = u8;

#[derive(Debug)]
pub struct System {
    registers: [Constant; 16],
    address_register: Address,
    stack: VecDeque<Register>,
    delay_timer: Constant,
    sound_timer: Constant,
    program_counter: Address,

    last_tick: Instant,
}

impl System {
    pub fn new() -> System {
        System {
            registers: [0; 16],
            address_register: 0x0,
            stack: VecDeque::new(),
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x0,

            last_tick: Instant::now()
        }
    }

    pub fn process_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Call(address) => {}
            Opcode::Clear => {}
            Opcode::Return => {}
            Opcode::Goto(address) => {}
            Opcode::CallFunction(address) => {}
            Opcode::SkipEq(register, constant) => {}
            Opcode::SkipNEq(register, constant) => {}
            Opcode::SkipEqReg(first, second) => {}
            Opcode::Set(register, constant) => {
                self.set_register(register, constant);
            }
            Opcode::AddAssign(register, constant) => {
                let value = self.get_register(register);
                self.set_register(register, register + constant);
            }
            Opcode::Copy(to, from) => {
                let from_value = self.get_register(from);
                self.set_register(to, from_value);
            }
            Opcode::Or(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value | second_value);
            }
            Opcode::And(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value & second_value);
            }
            Opcode::Xor(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value ^ second_value);
            }
            Opcode::AddAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);


            }
            Opcode::SubAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let borrow = if first_value > second_value {
                    1
                } else {
                    0
                };
            }
            Opcode::ShiftRight(first, second) => {
                let original_value = self.get_register(second);
                let lowest_bit = original_value & 0x1;

                let value = original_value >> 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(lowest_bit);
            }
            Opcode::Subtract(first, second) => {}
            Opcode::ShiftLeft(first, second) => {
                let original_value = self.get_register(second);
                let highest_bit = original_value & 0x8;

                let value = original_value << 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(highest_bit);
            }
            Opcode::SkipNEqReg(first, second) => {}
            Opcode::SetAddressReg(address) => {
                self.address_register = address;
            }
            Opcode::JumpOffset(constant) => {}
            Opcode::SetRand(register, constant) => {}
            Opcode::Draw(first, second, constant) => {}
            Opcode::SkipKeyPress(register) => {}
            Opcode::SkipNoKeyPress(register) => {}
            Opcode::StoreDelayTimer(register) => {
                let delay = self.delay_timer;
                self.set_register(register, delay);
            }
            Opcode::StoreKeypress(register) => {}
            Opcode::SetDelayTimer(register) => {
                self.delay_timer = self.get_register(register);
            }
            Opcode::SetSoundTimer(register) => {
                self.sound_timer = self.get_register(register);
            }
            Opcode::IncrementAddressReg(register) => {
                self.address_register += self.get_register(register) as u16;
            }
            Opcode::StoreSpriteAddress(register) => {}
            Opcode::BinaryCodedDecimal(register) => {}
            Opcode::Dump(register) => {
                for i in 0..(register + 1) {
                    let to_register = self.address_register as u8;
                    let value = self.get_register(i);

                    self.set_register(to_register, value);
                    self.address_register += 1;
                }
            }
            Opcode::Load(register) => {
                for i in 0..(register + 1) {
                    let from_register = self.address_register as u8;
                    let value = self.get_register(from_register);

                    self.set_register(i, value);
                    self.address_register += 1;
                }
            }
        }
    }

    pub fn tick(&mut self, rate: u8) {
        let elapsed = (self.last_tick.elapsed().as_secs() % (rate as u64)) as u8;

        if self.delay_timer > 0 {
            self.delay_timer -= elapsed;
        }

        if self.sound_timer > 0 {
            self.sound_timer -= elapsed;
        }

        self.last_tick = Instant::now();
    }

    fn get_register(&self, register: Register) -> Constant {
        self.registers[register as usize]
    }

    fn set_register(&mut self, register: Register, value: Constant) {
        self.registers[register as usize] = value;
    }

    fn set_flag_register(&mut self, value: Constant) {
        self.registers[15] = value;
    }
}