use std::{
    collections::VecDeque,
    thread,
    time::Duration,
};

use rand;
use rand::distributions::{IndependentSample, Range};
use sdl2;

use crate::{
    display::Display,
    input::{Input, InputAction},
    memory::{load_fonts, load_program},
    opcode::Opcode,
    sound::Sound,
};

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
    sound: Sound,

    rng: rand::ThreadRng,
}

impl System {
    pub fn new(program: &[u8]) -> System {
        let mut memory = [0; 4096];
        load_fonts(&mut memory);
        load_program(&mut memory, program);

        let sdl_context = sdl2::init().unwrap();
        let display = Display::new(&sdl_context);
        let input = Input::new(&sdl_context);
        let sound = Sound::new(&sdl_context);

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
            sound,

            rng: rand::thread_rng(),
        }
    }

    pub fn run(&mut self, dump_state: bool) -> Result<(), ()> {
        let mut tick_rate: Duration = Duration::from_millis(16);
        let mut running = true;
        let mut paused = false;

        while running {
            match self.input.handle_input() {
                Some(InputAction::Quit) => running = false,
                Some(InputAction::Reset) => {
                    // We're going to need to reset program memory as well

                    self.registers = [0; 16];
                    self.address_register = 0x0;
                    self.stack.clear();
                    self.delay_timer = 0;
                    self.sound_timer = 0;
                    self.program_counter = 0x200;

                    self.display.clear();
                }
                Some(InputAction::Pause) => paused = !paused,
                Some(InputAction::DecreaseTick) => {
                    if tick_rate.as_millis() >= 8 {
                        tick_rate -= Duration::from_millis(4);
                    } else {
                        tick_rate = Duration::from_millis(4);
                    }
                }
                Some(InputAction::IncreaseTick) => {
                    tick_rate += Duration::from_millis(4);
                }
                Some(InputAction::DebugInfo) => {
                    if !dump_state {
                        self.print_debug();
                    }
                }
                _ => {}
            }

            if paused {
                continue;
            }

            if dump_state {
                self.print_debug();
            }

            let first_address = self.program_counter as usize;
            let second_address = (self.program_counter + 1) as usize;

            let first_byte = self.memory[first_address];
            let second_byte = self.memory[second_address];

            if let Ok(opcode) = Opcode::from(first_byte, second_byte) {
                self.process_opcode(opcode)?;
            }

            self.tick(tick_rate);
        }

        Err(())
    }

    fn print_debug(&mut self) {
        print!("PC[{:#04x}]\tDELAY[{}]\tSOUND[{}]\tI[{:#03x}]", self.program_counter, self.delay_timer, self.sound_timer, self.address_register);
        for i in 0x0..0x10 {
            print!("\tV{:X}[{}]", i, self.get_register(i));
        }

        let first_address = self.program_counter as usize;
        let second_address = (self.program_counter + 1) as usize;

        let first_byte = self.memory[first_address];
        let second_byte = self.memory[second_address];

        let op = Opcode::from(first_byte, second_byte).unwrap();
        println!("\t{:?}", op);
    }

    fn process_opcode(&mut self, opcode: Opcode) -> Result<(), ()> {
        match opcode {
            Opcode::Call(_) => {
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

                let result = if let Some(result) = value.checked_add(constant) {
                    result
                } else {
                    (value as u16 + constant as u16) as u8
                };

                self.set_register(register, result);
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

                let result = if let Some(result) = first_value.checked_add(second_value) {
                    self.set_flag_register(0x0);
                    result
                } else {
                    self.set_flag_register(0x1);
                    (first_value as u16 + second_value as u16) as u8
                };

                self.set_register(first, result);


                self.program_counter += WORD_SIZE;
            }
            Opcode::SubAssignReg(first, second) => {
                let first_value = self.get_register(first);
                let second_value = self.get_register(second);

                if first_value >= second_value {
                    self.set_flag_register(0x1);
                } else {
                    self.set_flag_register(0x0);
                }

                self.set_register(first, first_value.wrapping_sub(second_value));

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

                if second_value >= first_value {
                    self.set_flag_register(0x1);
                } else {
                    self.set_flag_register(0x0);
                }

                self.set_register(first, second_value.wrapping_sub(first_value));

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
            Opcode::JumpOffset(address) => {
                self.program_counter = address + self.get_register(0x0) as u16;
            }
            Opcode::SetRand(register, constant) => {
                let range = Range::new(0, constant);
                let random_value = range.ind_sample(&mut self.rng);

                self.set_register(register, random_value);

                self.program_counter += WORD_SIZE;
            }
            Opcode::Draw(first, second, constant) => {
                let mut collision = false;

                let mut x = self.get_register(second);
                for i in self.address_register..(self.address_register + constant as u16) {
                    let sprite = self.get_memory(i);

                    let y = self.get_register(first);

                    collision |= self.display.draw(x, y, sprite);

                    if (x as usize) == self.display.screen_dimensions().0 - 1 {
                        x = 0;
                    } else {
                        x += 1;
                    }
                }

                if collision {
                    self.set_flag_register(0x1);
                } else {
                    self.set_flag_register(0x0);
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

                let ones = value / 100;
                let tens = (value / 10) % 10;
                let hundreds = (value % 100) % 10;

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

    fn tick(&mut self, tick_rate: Duration) {
        let mut active_delay = self.delay_timer > 0;
        let mut active_sound = self.sound_timer > 0;

        if active_sound {
            self.sound.play()
        }

        while active_delay | active_sound {
            if active_delay && self.delay_timer <= 0 {
                self.delay_timer = 0;
                active_delay = false;
            } else if active_delay {
                self.delay_timer -= 1;
            }

            if active_sound && self.sound_timer <= 0 {
                self.sound_timer = 0;
                self.sound.stop();
                active_sound = false;
            } else if active_sound {
                self.sound_timer -= 1;
            }
        }

        thread::sleep(tick_rate);
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