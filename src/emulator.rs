use rand;
use rand::distributions::{IndependentSample, Range};

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

            rng: rand::thread_rng(),
            last_tick: Instant::now()
        }
    }

    pub fn run(&mut self) -> Result<(), ()> {
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

    fn process_opcode(&mut self, opcode: Opcode) -> Result<(), ()> {
        match opcode {
            Opcode::Call(address) => {
                self.program_counter += WORD_SIZE;
            }
            Opcode::Clear => {
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
                self.program_counter += WORD_SIZE;
            }
            Opcode::SkipKeyPress(register) => {
                // TODO(Matt): Handle key events
                unimplemented!();
            }
            Opcode::SkipNoKeyPress(register) => {
                // TODO(Matt): Handle key events
                unimplemented!();
            }
            Opcode::StoreDelayTimer(register) => {
                let delay = self.delay_timer;
                self.set_register(register, delay);
                self.program_counter += WORD_SIZE;
            }
            Opcode::StoreKeypress(register) => {
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
                self.program_counter += WORD_SIZE;
            }
            Opcode::BinaryCodedDecimal(register) => {
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