use rand;
use sdl2;

use rand::distributions::{IndependentSample, Range};

use std::thread;
use std::time::Instant;
use std::collections::VecDeque;

use opcode::Opcode;
use display::Display;
use input::{Input, InputAction};
use memory::{load_fonts, load_program};

pub type Address = u16;
pub type Constant = u8;
pub type Register = u8;

const WORD_SIZE: u16 = 2;

pub struct System {
    memory: [Constant; 4096],
    registers: [Constant; 16],
    address_register: Address,
    stack: VecDeque<Address>,
    delay_timer: Constant,
    sound_timer: Constant,
    program_counter: Address,

    display: Display,
    input: Input,

    rng: rand::ThreadRng,
    last_tick: Instant,
}

impl System {
    pub fn new(program: &[u8]) -> System {
        let mut memory = [0; 4096];
        load_fonts(&mut memory);
        load_program(&mut memory, program);

        let sdl_context = sdl2::init().unwrap();
        let display = Display::new(&sdl_context);
        let input = Input::new(&sdl_context);

        System {
            memory,
            registers: [0; 16],
            address_register: 0x0,
            stack: VecDeque::with_capacity(24),
            delay_timer: 0,
            sound_timer: 0,
            program_counter: 0x200,

            display,
            input,

            rng: rand::thread_rng(),
            last_tick: Instant::now(),
        }
    }

    pub fn run_cli(&mut self) -> Result<(), ()> {
        println!("PC\tDELAY\tSOUND\tOP\tARG1\tARG2\tARG3");
        println!("--\t-----\t-----\t--\t----\t----\t----");
        loop {
            let first_address = self.program_counter as usize;
            let second_address = (self.program_counter + 1) as usize;

            let first_byte = self.memory[first_address];
            let second_byte = self.memory[second_address];

            print!("{:#04x}\t{}\t{}\t", self.program_counter, self.delay_timer, self.sound_timer);
            match Opcode::from(first_byte, second_byte) {
                Ok(opcode) => {
                    println!("{:?}", opcode);
                    self.process_opcode(opcode)?;
                }
                Err((first, second)) => {
                    println!("DATA\t{:x}{:x}", first.0, second.0);
                    self.program_counter += WORD_SIZE;
                }
            }

            self.tick(60);
        }
    }

    pub fn run_gui(&mut self) -> Result<(), ()> {
        let mut sleep_rate = 1;
        let mut running = true;
        while running {
            match self.input.handle_input() {
                Some(InputAction::Quit) => running = false,
                Some(InputAction::DecreaseTick) => {
                    if sleep_rate >= 10 {
                        sleep_rate -= 10;
                    } else {
                        sleep_rate = 1;
                    }
                }
                Some(InputAction::IncreaseTick) => {
                    sleep_rate += 10;
                }
                _ => {}
            }

            let first_address = self.program_counter as usize;
            let second_address = (self.program_counter + 1) as usize;

            let first_byte = self.memory[first_address];
            let second_byte = self.memory[second_address];

            if let Ok(opcode) = Opcode::from(first_byte, second_byte) {
                self.process_opcode(opcode)?;
            }

            self.tick(60);
            thread::sleep_ms(sleep_rate);
        }

        Err(())
    }

    fn process_opcode(&mut self, opcode: Opcode) -> Result<(), ()> {
        match opcode {
            Opcode::Call(address) => {
                self.program_counter += WORD_SIZE;
            }
            Opcode::Clear => {
                self.display.clear();
                self.program_counter += WORD_SIZE;
            }
            Opcode::Return => {
                if let Some(address) = self.stack.pop_front() {
                    self.program_counter = address + WORD_SIZE;
                } else {
                    println!("NOWHERE TO RETURN");
                    return Err(());
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
                    self.program_counter += 2 * WORD_SIZE;
                } else {
                    self.program_counter += WORD_SIZE;
                }
            }
            Opcode::SkipNEq(register, constant) => {
                if self.get_register(register) != constant {
                    self.program_counter += 2 * WORD_SIZE;
                } else {
                    self.program_counter += WORD_SIZE;
                }
            }
            Opcode::SkipEqReg(first, second) => {
                if self.get_register(first) == self.get_register(second) {
                    self.program_counter += 2 * WORD_SIZE;
                } else {
                    self.program_counter += WORD_SIZE;
                }
            }
            Opcode::Set(register, constant) => {
                self.set_register(register, constant);
                self.program_counter += WORD_SIZE;
            }
            Opcode::AddAssign(register, constant) => {
                let value = self.get_register(register);
                self.set_register(register, value.wrapping_add(constant));
                self.program_counter += WORD_SIZE;
            }
            Opcode::Copy(to, from) => {
                let from_value = self.get_register(from);
                self.set_register(to, from_value);
                self.program_counter += WORD_SIZE;
            }
            Opcode::Or(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value | second_value);
                self.program_counter += WORD_SIZE;
            }
            Opcode::And(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value & second_value);
                self.program_counter += WORD_SIZE;
            }
            Opcode::Xor(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);
                self.set_register(first, first_value ^ second_value);
                self.program_counter += WORD_SIZE;
            }
            Opcode::AddAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = first_value.overflowing_add(second_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += WORD_SIZE;
            }
            Opcode::SubAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = first_value.overflowing_sub(second_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += WORD_SIZE;
            }
            Opcode::ShiftRight(first, second) => {
                let original_value = self.get_register(second);
                let lowest_bit = original_value & 0x1;

                let value = original_value >> 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(lowest_bit);

                self.program_counter += WORD_SIZE;
            }
            Opcode::Subtract(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                let (result, carry) = second_value.overflowing_sub(first_value);
                self.set_register(first, result);
                self.set_flag_register(if carry { 1 } else { 0 });

                self.program_counter += WORD_SIZE;
            }
            Opcode::ShiftLeft(first, second) => {
                let original_value = self.get_register(second);
                let highest_bit = original_value & 0x8;

                let value = original_value << 1;

                self.set_register(first, value);
                self.set_register(second, value);
                self.set_flag_register(highest_bit);

                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipNEqReg(first, second) => {
                if self.get_register(first) != self.get_register(second) {
                    self.program_counter += 2 * WORD_SIZE;
                } else {
                    self.program_counter += WORD_SIZE;
                }
            }
            Opcode::SetAddressReg(address) => {
                self.address_register = address;
                self.program_counter += WORD_SIZE;
            }
            Opcode::JumpOffset(constant) => {
                self.program_counter = self.get_register(0x0) as u16 + self.address_register;
            }
            Opcode::SetRand(register, constant) => {
                let range = Range::new(0, constant);
                let random_value = range.ind_sample(&mut self.rng);

                self.set_register(register, constant);

                self.program_counter += WORD_SIZE;
            }
            Opcode::Draw(first, second, constant) => {
                // TODO(Matt): This kinda sucks... Let's not do it bad next time, okay?
                let mut x = self.get_register(second);
                for i in self.address_register..(self.address_register + constant as u16) {
                    let mut sprite = self.get_memory(i);

                    let mut y = self.get_register(first);

                    self.display.draw(x, y, sprite);

                    if (x as usize) == self.display.screen_dimensions().0 - 1 {
                        x = 0;
                    } else {
                        x += 1;
                    }
                }

                self.display.render();

                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipKeyPress(register) => {
                let expected_key = self.get_register(register);
                if let Some(key) = self.input.key_pressed() {
                    if expected_key == key {
                        self.program_counter += 2 * WORD_SIZE;
                        return Ok(());
                    }
                }

                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipNoKeyPress(register) => {
                let expected_key = self.get_register(register);
                if let Some(key) = self.input.key_pressed() {
                    if expected_key != key {
                        self.program_counter += 2 * WORD_SIZE;
                        return Ok(());
                    }
                }

                self.program_counter += WORD_SIZE;
            }
            Opcode::StoreDelayTimer(register) => {
                let delay = self.delay_timer;
                self.set_register(register, delay);
                self.program_counter += WORD_SIZE;
            }
            Opcode::StoreKeypress(register) => {
                let pressed_key = self.input.get_key();
                self.set_register(register, pressed_key);
                self.program_counter += WORD_SIZE;
            }
            Opcode::SetDelayTimer(register) => {
                self.delay_timer = self.get_register(register);
                self.program_counter += WORD_SIZE;
            }
            Opcode::SetSoundTimer(register) => {
                self.sound_timer = self.get_register(register);
                self.program_counter += WORD_SIZE;
            }
            Opcode::IncrementAddressReg(register) => {
                self.address_register += self.get_register(register) as u16;
                self.program_counter += WORD_SIZE;
            }
            Opcode::StoreSpriteAddress(register) => {
                let value = self.get_register(register) as u16;
                self.address_register = match value {
                    0x0 => 0x000,
                    0x1 => 0x005,
                    0x2 => 0x00A,
                    0x3 => 0x00F,

                    0x4 => 0x014,
                    0x5 => 0x019,
                    0x6 => 0x024,
                    0x7 => 0x029,

                    0x8 => 0x034,
                    0x9 => 0x039,
                    0xA => 0x044,
                    0xB => 0x049,

                    0xC => 0x054,
                    0xD => 0x059,
                    0xE => 0x064,
                    0xF => 0x069,
                    _ => 0x000,
                };
                self.program_counter += WORD_SIZE;
            }
            Opcode::BinaryCodedDecimal(register) => {
                let memory_location = self.address_register;
                let value = self.get_register(register) as u16;

                let ones = value & 0xF;
                let tens = (value & 0xF0) >> 4;
                let hundreds = (value & 0xF00) >> 8;

                self.set_memory(memory_location, ones as u8);
                self.set_memory(memory_location + 1, tens as u8);
                self.set_memory(memory_location + 2, hundreds as u8);

                self.program_counter += WORD_SIZE;
            }
            Opcode::Dump(register) => {
                for i in 0..(register + 1) {
                    let memory_location = self.address_register;
                    let value = self.get_register(i);

                    self.set_memory(memory_location, value);
                    self.address_register += 1;
                }
                self.program_counter += WORD_SIZE;
            }
            Opcode::Load(register) => {
                for i in 0..(register + 1) {
                    let memory_location = self.address_register;
                    let value = self.get_memory(memory_location);

                    self.set_register(i, value);
                    self.address_register += 1;
                }
                self.program_counter += WORD_SIZE;
            }
        }

        Ok(())
    }

    fn tick(&mut self, rate: u8) {
        let elapsed = (self.last_tick.elapsed().subsec_nanos() % (rate as u32)) as u8;

        if self.delay_timer >= elapsed {
            self.delay_timer -= elapsed;
        } else {
            self.delay_timer = 0;
        }

        if self.sound_timer >= elapsed {
            self.sound_timer -= elapsed;
        } else {
            self.sound_timer = 0;
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
        self.registers[0xF] = value;
    }

    fn get_memory(&mut self, address: Address) -> Constant {
        self.memory[address as usize]
    }

    fn set_memory(&mut self, address: Address, value: Constant) {
        self.memory[address as usize] = value;
    }
}