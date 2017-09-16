use std::time::Instant;
use std::collections::VecDeque;

use opcode::Opcode;

pub type Address = u16;
pub type Constant = u8;
pub type Register = u8;

pub struct System {
    memory: [Constant; 4096],
    registers: [Constant; 16],
    address_register: Address,
    stack: VecDeque<Address>,
    delay_timer: Constant,
    sound_timer: Constant,
    program_counter: Address,

    last_tick: Instant,
}

impl System {
    pub fn new(program: &[u8]) -> System {
        let mut memory = [0; 4096];

        let mut current_address = 0x200;
        for byte in program {
            if current_address == 0xEA0 {
                break;
            }

            memory[current_address] = *byte;
            current_address += 1;
        }

        System {
            memory,
            registers: [0; 16],
            address_register: 0x0,
            stack: VecDeque::with_capacity(24),
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,

            last_tick: Instant::now()
        }
    }

    pub fn run(&mut self) {
        println!("PC\tOP\tARG1\tARG2\tARG3");
        println!("--\t--\t----\t----\t----");
        loop {
            let first_address = self.program_counter as usize;
            let second_address = (self.program_counter + 1) as usize;

            let first_byte = self.memory[first_address];
            let second_byte = self.memory[second_address];

            print!("{:x}\t", self.program_counter);
            if let Some(opcode) = Opcode::from(first_byte, second_byte) {
                println!("{:?}", opcode);
                self.process_opcode(opcode);
            } else {
                println!("LBL\t{:x}{:x}", first_byte, second_byte);
                self.program_counter += 2;
            }

            self.tick(60);
        }
    }

    fn process_opcode(&mut self, opcode: Opcode) {
        match opcode {
            Opcode::Call(address) => {
                self.program_counter += 2;
            }
            Opcode::Clear => {
                self.program_counter += 2;
            }
            Opcode::Return => {
                if let Some(address) = self.stack.pop_front() {
                    self.program_counter = address;
                } else {
                    println!("NOWHERE TO RETURN");
                    self.program_counter += 2;
                }
            }
            Opcode::Goto(address) => {
                self.program_counter = address;
            }
            Opcode::CallFunction(address) => {
                self.stack.push_front(self.program_counter);
                self.program_counter = address;
            }
            Opcode::SkipEq(register, constant) => {
                if self.get_register(register) == constant {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            Opcode::SkipNEq(register, constant) => {
                if self.get_register(register) != constant {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            Opcode::SkipEqReg(first, second) => {
                if self.get_register(first) == self.get_register(second) {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            Opcode::Set(register, constant) => {
                self.set_register(register, constant);
                self.program_counter += 2;
            }
            Opcode::AddAssign(register, constant) => {
                let value = self.get_register(register);
                self.set_register(register, register.wrapping_add(constant));
                self.program_counter += 2;
            }
            Opcode::Copy(to, from) => {
                let from_value = self.get_register(from);
                self.set_register(to, from_value);
                self.program_counter += 2;
            }
            Opcode::Or(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value | second_value);
                self.program_counter += 2;
            }
            Opcode::And(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value & second_value);
                self.program_counter += 2;
            }
            Opcode::Xor(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value ^ second_value);
                self.program_counter += 2;
            }
            Opcode::AddAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = first_value.overflowing_add(second_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += 2;
            }
            Opcode::SubAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = first_value.overflowing_sub(second_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += 2;
            }
            Opcode::ShiftRight(first, second) => {
                let original_value = self.get_register(second);
                let lowest_bit = original_value & 0x1;

                let value = original_value >> 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(lowest_bit);

                self.program_counter += 2;
            }
            Opcode::Subtract(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = second_value.overflowing_sub(first_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += 2;
            }
            Opcode::ShiftLeft(first, second) => {
                let original_value = self.get_register(second);
                let highest_bit = original_value & 0x8;

                let value = original_value << 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(highest_bit);

                self.program_counter += 2;
            }
            Opcode::SkipNEqReg(first, second) => {
                if self.get_register(first) != self.get_register(second) {
                    self.program_counter += 4;
                } else {
                    self.program_counter += 2;
                }
            }
            Opcode::SetAddressReg(address) => {
                self.address_register = address;
                self.program_counter += 2;
            }
            Opcode::JumpOffset(constant) => {
                self.program_counter = self.get_register(0x0) as u16 + self.address_register;
            }
            Opcode::SetRand(register, constant) => {
                self.program_counter += 2;
            }
            Opcode::Draw(first, second, constant) => {
                self.program_counter += 2;
            }
            Opcode::SkipKeyPress(register) => {}
            Opcode::SkipNoKeyPress(register) => {}
            Opcode::StoreDelayTimer(register) => {
                let delay = self.delay_timer;
                self.set_register(register, delay);
                self.program_counter += 2;
            }
            Opcode::StoreKeypress(register) => {
                self.program_counter += 2;
            }
            Opcode::SetDelayTimer(register) => {
                self.delay_timer = self.get_register(register);
                self.program_counter += 2;
            }
            Opcode::SetSoundTimer(register) => {
                self.sound_timer = self.get_register(register);
                self.program_counter += 2;
            }
            Opcode::IncrementAddressReg(register) => {
                self.address_register += self.get_register(register) as u16;
                self.program_counter += 2;
            }
            Opcode::StoreSpriteAddress(register) => {
                self.program_counter += 2;
            }
            Opcode::BinaryCodedDecimal(register) => {
                self.program_counter += 2;
            }
            Opcode::Dump(register) => {
                for i in 0..(register + 1) {
                    let to_register = self.address_register as u8;
                    let value = self.get_register(i);

                    self.set_register(to_register, value);
                    self.address_register += 1;
                }
                self.program_counter += 2;
            }
            Opcode::Load(register) => {
                for i in 0..(register + 1) {
                    let from_register = self.address_register as u8;
                    let value = self.get_register(from_register);

                    self.set_register(i, value);
                    self.address_register += 1;
                }
                self.program_counter += 2;
            }
        }
    }

    fn tick(&mut self, rate: u8) {
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