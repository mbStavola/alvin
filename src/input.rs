use sdl2::{
    self,
    event::Event,
    EventPump,
    keyboard::Keycode,
};

pub struct Input {
    event_pump: EventPump
}

impl Input {
    pub fn new(sdl_context: &sdl2::Sdl) -> Input {
        let event_pump = sdl_context.event_pump().unwrap();

        Input {
            event_pump
        }
    }

    pub fn handle_input(&mut self) -> Option<InputAction> {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    return Some(InputAction::Quit);
                }
                Event::Quit { .. } | Event::KeyUp { keycode: Some(Keycode::Escape), .. } => {
                    return Some(InputAction::Quit);
                }
                Event::KeyDown { keycode: Some(Keycode::Return), .. } => {
                    return Some(InputAction::Reset);
                }
                Event::KeyUp { keycode: Some(Keycode::Return), .. } => {
                    return Some(InputAction::Reset);
                }
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    return Some(InputAction::Pause);
                }
                Event::KeyDown { keycode: Some(Keycode::LeftBracket), .. } => {
                    return Some(InputAction::DecreaseTick);
                }
                Event::KeyDown { keycode: Some(Keycode::RightBracket), .. } => {
                    return Some(InputAction::IncreaseTick);
                }
                Event::KeyDown { keycode: Some(Keycode::Tab), .. } => {
                    return Some(InputAction::DebugInfo);
                }
                Event::KeyUp { keycode: Some(Keycode::Tab), .. } => {
                    return Some(InputAction::DebugInfo);
                }
                _ => {}
            }
        }

        None
    }

    pub fn key_pressed(&mut self) -> Option<u8> {
        for event in self.event_pump.wait_timeout_iter(10) {
            match event {
                Event::KeyDown { keycode, .. } | Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        return key_map(key);
                    }
                }
                _ => {}
            }
        }

        None
    }

    pub fn get_key(&mut self) -> u8 {
        loop {
            match self.event_pump.wait_event() {
                Event::KeyDown { keycode, .. } | Event::KeyUp { keycode, .. } => {
                    if let Some(key) = keycode {
                        if let Some(key_constant) = key_map(key) {
                            return key_constant;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

pub enum InputAction {
    Quit,
    Reset,
    Pause,
    DecreaseTick,
    IncreaseTick,
    DebugInfo,
}

pub fn key_map(keycode: Keycode) -> Option<u8> {
    return match keycode {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),

        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),

        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),

        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None
    };
}