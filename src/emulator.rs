use rand;
use sdl2;
use sdl2::EventPump;
use sdl2::rect::Rect;
use sdl2::rect::Point;
use sdl2::event::Event;
use sdl2::pixels::Color;
use std::time::Duration;
use sdl2::video::Window;
use sdl2::render::Canvas;
use sdl2::keyboard::Keycode;
use rand::distributions::{IndependentSample, Range};

use std::thread;
use std::time::Instant;
use std::collections::VecDeque;

use opcode::Opcode;

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
    display: [[bool; 32]; 64],

    rng: rand::ThreadRng,
    last_tick: Instant,
}

impl System {
    pub fn new(program: &[u8]) -> System {
        let mut memory = [0; 4096];
        load_fonts(&mut memory);

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
            display: [[false; 32]; 64],

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
                    self.process_opcode(opcode, None, None)?;
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
        let sdl_context = sdl2::init().unwrap();
        let video_subsystem = sdl_context.video().unwrap();

        let window = video_subsystem.window("Alvin", 640, 320)
            .position_centered().build().unwrap();

        let mut canvas = window.into_canvas().accelerated().build().unwrap();
        canvas.set_scale(10.0, 10.0);
        canvas.set_draw_color(BG_COLOR);
        canvas.clear();

        let mut event_pump = sdl_context.event_pump().unwrap();

        let mut running = true;
        while running {
            for event in event_pump.poll_iter() {
                match event {
                    Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        running = false;
                    }
                    _ => {}
                }
            }

            let first_address = self.program_counter as usize;
            let second_address = (self.program_counter + 1) as usize;

            let first_byte = self.memory[first_address];
            let second_byte = self.memory[second_address];

            if let Ok(opcode) = Opcode::from(first_byte, second_byte) {
                self.process_opcode(opcode, Some(&mut event_pump), Some(&mut canvas))?;
            }

            self.tick(60);
            thread::sleep_ms(1);
        }

        Err(())
    }

    fn process_opcode(&mut self, opcode: Opcode, event_pump: Option<&mut EventPump>, canvas: Option<&mut Canvas<Window>>) -> Result<(), ()> {
        match opcode {
            Opcode::Call(address) => {
                self.program_counter += WORD_SIZE;
            }
            Opcode::Clear => {
                if let Some(canvas) = canvas {
                    canvas.set_draw_color(BG_COLOR);
                    self.display = [[false; 32]; 64];
                    canvas.clear();
                    canvas.present();
                }

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
                if let Some(canvas) = canvas {
                    let mut x = self.get_register(second) as usize;
                    for i in self.address_register..(self.address_register + constant as u16) {
                        let mut sprite = self.get_memory(i);

                        let mut y = self.get_register(first) as usize;

                        for _ in 0x0..0x8 {
                            let highest_bit = (sprite & 0x80) == 0x80;
                            sprite = sprite << 1;

                            if self.display[y][x] & highest_bit {
                                self.set_flag_register(0x1);
                            }

                            self.display[y][x] ^= highest_bit;

                            if y == self.display.len() - 1 {
                                y = 0;
                            } else {
                                y += 1;
                            }
                        }

                        if x == self.display[y].len() - 1 {
                            x = 0;
                        } else {
                            x += 1;
                        }
                    }

                    for i in 0..self.display.len() {
                        for j in 0..self.display[i].len() {
                            let color = if self.display[i][j] {
                                FG_COLOR
                            } else {
                                BG_COLOR
                            };

                            canvas.set_draw_color(color);
                            canvas.draw_point(Point::new(i as i32, j as i32));
                        }
                    }

                    canvas.present();
                }

                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipKeyPress(register) => {
                if let Some(event_pump) = event_pump {
                    match event_pump.poll_event() {
                        Some(Event::KeyDown { keycode, .. }) => {
                            if let Some(keycode) = keycode {
                                let expected = self.get_register(register) as i32;

                                if expected == key_map(keycode) {
                                    self.program_counter += 2 * WORD_SIZE;
                                    return Ok(());
                                }
                            }
                        }
                        _ => {}
                    }
                }

                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipNoKeyPress(register) => {
                if let Some(event_pump) = event_pump {
                    match event_pump.poll_event() {
                        Some(Event::KeyDown { keycode, .. }) => {
                            if let Some(keycode) = keycode {
                                let expected = self.get_register(register) as i32;

                                if expected != key_map(keycode) {
                                    self.program_counter += 2 * WORD_SIZE;
                                    return Ok(());
                                }
                            }
                        }
                        _ => {}
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
                if let Some(event_pump) = event_pump {
                    loop {
                        match event_pump.wait_event() {
                            Event::KeyDown { keycode, .. } => {
                                if let Some(keycode) = keycode {
                                    self.set_register(register, key_map(keycode) as u8);
                                    self.program_counter += WORD_SIZE;
                                    return Ok(());
                                }
                            }
                            _ => {}
                        }
                    }
                }
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
        self.registers[15] = value;
    }

    fn get_memory(&mut self, address: Address) -> Constant {
        self.memory[address as usize]
    }

    fn set_memory(&mut self, address: Address, value: Constant) {
        self.memory[address as usize] = value;
    }
}

const BG_COLOR: Color = Color { r: 53, g: 59, b: 115, a: 0xFF };
const FG_COLOR: Color = Color { r: 255, g: 255, b: 41, a: 0xFF };

fn key_map(keycode: Keycode) -> i32 {
    match keycode {
        Keycode::Num1 => 0x1,
        Keycode::Num2 => 0x2,
        Keycode::Num3 => 0x3,
        Keycode::Num4 => 0xC,

        Keycode::Q => 0x4,
        Keycode::W => 0x5,
        Keycode::E => 0x6,
        Keycode::R => 0xD,

        Keycode::A => 0x7,
        Keycode::S => 0x8,
        Keycode::D => 0x9,
        Keycode::F => 0xE,

        Keycode::Z => 0xA,
        Keycode::X => 0x0,
        Keycode::C => 0xB,
        Keycode::V => 0xF,
        _ => -1
    }
}

fn load_fonts(memory: &mut [u8]) {
    let mut i = 0x0;
    for sprite in SPRITE_DATA.iter() {
        memory[i] = *sprite;
        i += 1;
    }
}

// Sprite data borrowed from https://github.com/massung/CHIP-8/blob/master/chip8/rom.go
const SPRITE_DATA: [u8; 0x1C0] = [
    // 4x5 low-res mode font sprites (0-F)
    0xF0, 0x90, 0x90, 0x90, 0xF0, 0x20, 0x60, 0x20,
    0x20, 0x70, 0xF0, 0x10, 0xF0, 0x80, 0xF0, 0xF0,
    0x10, 0xF0, 0x10, 0xF0, 0xA0, 0xA0, 0xF0, 0x20,
    0x20, 0xF0, 0x80, 0xF0, 0x10, 0xF0, 0xF0, 0x80,
    0xF0, 0x90, 0xF0, 0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0, 0xF0, 0x90, 0xF0,
    0x10, 0xF0, 0xF0, 0x90, 0xF0, 0x90, 0x90, 0xE0,
    0x90, 0xE0, 0x90, 0xE0, 0xF0, 0x80, 0x80, 0x80,
    0xF0, 0xE0, 0x90, 0x90, 0x90, 0xE0, 0xF0, 0x80,
    0xF0, 0x80, 0xF0, 0xF0, 0x80, 0xF0, 0x80, 0x80,
    // 8x10 high-res mode font sprites (0-F)
    0x3C, 0x7E, 0xE7, 0xC3, 0xC3, 0xC3, 0xC3, 0xE7,
    0x7E, 0x3C, 0x18, 0x38, 0x58, 0x18, 0x18, 0x18,
    0x18, 0x18, 0x18, 0x3C, 0x3E, 0x7F, 0xC3, 0x06,
    0x0C, 0x18, 0x30, 0x60, 0xFF, 0xFF, 0x3C, 0x7E,
    0xC3, 0x03, 0x0E, 0x0E, 0x03, 0xC3, 0x7E, 0x3C,
    0x06, 0x0E, 0x1E, 0x36, 0x66, 0xC6, 0xFF, 0xFF,
    0x06, 0x06, 0xFF, 0xFF, 0xC0, 0xC0, 0xFC, 0xFE,
    0x03, 0xC3, 0x7E, 0x3C, 0x3E, 0x7C, 0xC0, 0xC0,
    0xFC, 0xFE, 0xC3, 0xC3, 0x7E, 0x3C, 0xFF, 0xFF,
    0x03, 0x06, 0x0C, 0x18, 0x30, 0x60, 0x60, 0x60,
    0x3C, 0x7E, 0xC3, 0xC3, 0x7E, 0x7E, 0xC3, 0xC3,
    0x7E, 0x3C, 0x3C, 0x7E, 0xC3, 0xC3, 0x7F, 0x3F,
    0x03, 0x03, 0x3E, 0x7C, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    // 6-bit ASCII character patterns
    0x00, // |        |
    0x10, // |   *    |
    0x20, // |  *     |
    0x88, // |*   *   |
    0xA8, // |* * *   |
    0x50, // | * *    |
    0xF8, // |*****   |
    0x70, // | ***    |
    0x80, // |*       |
    0x90, // |*  *    |
    0xA0, // |* *     |
    0xB0, // |* **    |
    0xC0, // |**      |
    0xD0, // |** *    |
    0xE0, // |***     |
    0xF0, // |****    |

    // 6-bit ASCII characters from 0x100-
    0x46, 0x3E, 0x56, // @
    0x99, 0x9F, 0x4F, // A
    0x5F, 0x57, 0x4F, // B
    0x8F, 0x88, 0x4F, // C
    0x5F, 0x55, 0x4F, // D
    0x8F, 0x8F, 0x4F, // E
    0x88, 0x8F, 0x4F, // F
    0x9F, 0x8B, 0x4F, // G
    0x99, 0x9F, 0x49, // H
    0x27, 0x22, 0x47, // I
    0xAE, 0x22, 0x47, // J
    0xA9, 0xAC, 0x49, // K
    0x8F, 0x88, 0x48, // L
    0x43, 0x64, 0x53, // M
    0x99, 0xDB, 0x49, // N
    0x9F, 0x99, 0x4F, // O
    0x88, 0x9F, 0x4F, // P
    0x9F, 0x9B, 0x4F, // Q
    0xA9, 0x9F, 0x4F, // R
    0x1F, 0x8F, 0x4F, // S
    0x22, 0x22, 0x56, // T
    0x9F, 0x99, 0x49, // U
    0x22, 0x55, 0x53, // V
    0x55, 0x44, 0x54, // W
    0x53, 0x52, 0x53, // X
    0x22, 0x52, 0x53, // Y
    0xCF, 0x12, 0x4F, // Z
    0x8C, 0x88, 0x3C, // [
    0x10, 0xC2, 0x40, // \
    0x2E, 0x22, 0x3E, // ]
    0x30, 0x25, 0x50, // ^
    0x06, 0x00, 0x50, // _
    0x00, 0x00, 0x40, // space
    0x0C, 0xCC, 0x2C, // !
    0x00, 0x50, 0x45, // "
    0x65, 0x65, 0x55, // #
    0x46, 0x46, 0x56, // $
    0xDF, 0xBF, 0x4F, // %
    0x5F, 0xAF, 0x4E, // &
    0x00, 0x80, 0x18, // '
    0x21, 0x22, 0x41, // (
    0x12, 0x11, 0x42, // )
    0x53, 0x56, 0x53, // *
    0x22, 0x26, 0x52, // +
    0x2E, 0x00, 0x30, // ,
    0x00, 0x06, 0x50, // -
    0xCC, 0x00, 0x20, // .
    0xC0, 0x12, 0x40, // /
    0x9F, 0x99, 0x4F, // 0
    0x22, 0x22, 0x32, // 1
    0x8F, 0x1F, 0x4F, // 2
    0x1F, 0x1F, 0x4F, // 3
    0x22, 0xAF, 0x4A, // 4
    0x1F, 0x8F, 0x4F, // 5
    0x9F, 0x8F, 0x4F, // 6
    0x11, 0x11, 0x4F, // 7
    0x9F, 0x9F, 0x4F, // 8
    0x1F, 0x9F, 0x4F, // 9
    0x80, 0x80, 0x10, // :
    0x2E, 0x20, 0x30, // ;
    0x21, 0x2C, 0x41, // <
    0xE0, 0xE0, 0x30, // =
    0x2C, 0x21, 0x4C, // >
    0x88, 0x1F, 0x4F, // ?
];