use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::EventPump;

pub struct Input {
    event_pump: EventPump,
    should_exit: bool,
    key_states: [bool; 16],
}

fn convert_keycode_to_u8(keycode: Keycode) -> Option<u8> {
    match keycode {
        Keycode::Num1 => Some(0x0),
        Keycode::Num2 => Some(0x1),
        Keycode::Num3 => Some(0x2),
        Keycode::Num4 => Some(0x3),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0x7),
        Keycode::A => Some(0x8),
        Keycode::S => Some(0x9),
        Keycode::D => Some(0xa),
        Keycode::F => Some(0xb),
        Keycode::Y => Some(0xc),
        Keycode::X => Some(0xd),
        Keycode::C => Some(0xe),
        Keycode::V => Some(0xf),
        _ => None,
    }
}

impl Input {
    pub fn init(sdl_context: sdl2::Sdl) -> Input {
        Input {
            event_pump: sdl_context
                .event_pump()
                .expect("ERROR: Could not extract event-pump from sdl-context. Exiting..."),
            should_exit: false,
            key_states: [false; 16],
        }
    }

    pub fn is_key_pressed(&self, key_code: u8) -> bool {
        self.key_states[key_code as usize]
    }

    pub fn get_key_blocking(&mut self) -> u8 {
        loop {
            for event in self.event_pump.poll_iter() {
                if let Event::KeyDown { keycode, .. } = event {
                    if let Some(v) = keycode {
                        if let Some(v) = convert_keycode_to_u8(v) {
                            return v;
                        }
                    }
                    // TODO handle Event::Quit here!
                }
            }

            std::thread::sleep(std::time::Duration::new(0, 20_000));
        }
    }

    pub fn should_exit(&self) -> bool {
        self.should_exit
    }

    pub fn process_all_events(&mut self) {
        for event in self.event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => {
                    self.should_exit = true;
                }
                Event::KeyDown { keycode, .. } => {
                    if let Some(v) = keycode {
                        if let Some(v) = convert_keycode_to_u8(v) {
                            self.key_states[v as usize] = true;
                        }
                    }
                }
                Event::KeyUp { keycode, .. } => {
                    if let Some(v) = keycode {
                        if let Some(v) = convert_keycode_to_u8(v) {
                            self.key_states[v as usize] = false;
                        }
                    }
                }
                _ => (),
            }
        }
    }
}
