use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;

use display::Display;
use input::Input;
use sound::Sound;

mod display;
mod input;
mod sound;

const MEMORY_SIZE: u16 = 4096;
const PROGRAM_OFFSET: u16 = 0x200;
const FONT_STARTING_MEMORY: u16 = 0x050;

const FONTS: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

type Intermediate = u8;
type Address = u16;
type RegisterIdentifier = u8;
type Register = u8;

type AtomicRegister = AtomicU8;

// this follows the wikipedia article to chip8,
// meaning not the original CHIP8 instruction-set
#[derive(Debug)]
enum Instructions {
    ClearScreen,
    DrawSprite(RegisterIdentifier, RegisterIdentifier, Intermediate),
    UnconditionalJump(Address),
    UnconditionalJumpWithOffset(Address),
    SetVxToIntermediate(RegisterIdentifier, Intermediate),
    AddIntermediateToVx(RegisterIdentifier, Intermediate),
    SetIndexRegisterToIntermediate(Address),
    SkipIfKeyPressedVx(RegisterIdentifier),
    SkipIfKeyNotPressedVx(RegisterIdentifier),
    AwaitKeyPressVx(RegisterIdentifier),
    ReturnFromSubroutine,
    CallSubroutine(Address),
    SkipIfVxIsIntermediate(RegisterIdentifier, Intermediate),
    SkipIfVxIsNotIntermediate(RegisterIdentifier, Intermediate),
    SkipIfVxIsNotVy(RegisterIdentifier, RegisterIdentifier),
    SkipIfVxIsVy(RegisterIdentifier, RegisterIdentifier),
    SetVxToVy(RegisterIdentifier, RegisterIdentifier),
    BitwiseOrVyToVx(RegisterIdentifier, RegisterIdentifier),
    BitwiseAndVyToVx(RegisterIdentifier, RegisterIdentifier),
    BitwiseXorVyToVx(RegisterIdentifier, RegisterIdentifier),
    AddVyToVx(RegisterIdentifier, RegisterIdentifier),
    SubtractVyFromVx(RegisterIdentifier, RegisterIdentifier),
    StoreLSBfromVxInVf(RegisterIdentifier),
    StoreMSBfromVxInVf(RegisterIdentifier),
    SetVxToVyMinusVx(RegisterIdentifier, RegisterIdentifier),
    GenerateRandomNumberWithCap(RegisterIdentifier, Intermediate),
    SetVxToDelayTimer(RegisterIdentifier),
    SetDelayTimerToVx(RegisterIdentifier),
    SetSoundTimerToVx(RegisterIdentifier),
    AddVxToI(RegisterIdentifier),
    SetIToSpriteLocation(RegisterIdentifier),
    StoreVxAsBCDInI(RegisterIdentifier),
    DumpRegisters(RegisterIdentifier),
    LoadRegisters(RegisterIdentifier),
    Unkown,
}

pub struct Chip8 {
    data_registers: [Register; 16],
    memory: [u8; MEMORY_SIZE as usize],
    program_counter: Address,
    index_register: Address,
    stack: Vec<Address>,
    delay_timer: Arc<AtomicRegister>,
    sound_timer: Arc<AtomicRegister>,
    thread_killer: Arc<AtomicBool>,
    timer_thread: Option<JoinHandle<()>>,
    display: Display,
    input: Input,
    sound: Sound,
}

impl Chip8 {
    pub fn init() -> Chip8 {
        // initialize sdl
        let sdl_context = sdl2::init().expect("ERROR: Unable to initialize SDL. Exiting...");

        let sound_timer = Arc::new(AtomicU8::new(0));

        let mut chip = Chip8 {
            data_registers: [0; 16],
            memory: [0; MEMORY_SIZE as usize],
            program_counter: 0x00,
            index_register: 0x00,
            stack: Vec::new(),
            delay_timer: Arc::new(AtomicU8::new(0)),
            sound_timer: sound_timer.clone(),
            thread_killer: Arc::new(AtomicBool::new(false)),
            timer_thread: None,
            display: Display::init(sdl_context.clone()),
            input: Input::init(sdl_context.clone()),
            sound: Sound::init(&sdl_context, sound_timer),
        };

        chip.setup_fonts();

        chip
    }

    pub fn draw_display(&mut self) {
        self.display.draw();
    }

    fn setup_fonts(&mut self) {
        for i in 0..FONTS.len() {
            self.memory[FONT_STARTING_MEMORY as usize + i] = FONTS[i];
        }
    }

    pub fn start_sound_system(&self) {
        self.sound.start_sound_system();
    }

    pub fn stop_sound_system(&self) {
        self.sound.stop_sound_system();
    }

    pub fn stop_timers(&mut self) {
        self.thread_killer.store(true, Ordering::Relaxed);
        self.timer_thread.take().map(JoinHandle::join);
    }

    pub fn start_timers(&mut self) {
        let thread_killer = self.thread_killer.clone();
        let delay_timer = self.delay_timer.clone();
        let sound_timer = self.sound_timer.clone();

        self.timer_thread = Some(std::thread::spawn(move || {
            while !thread_killer.load(Ordering::Relaxed) {
                // TODO: these operations are not atomic. For now ignore this...
                // TODO: but you will want to use something like fetch_update...
                if delay_timer.load(Ordering::Relaxed) > 0 {
                    delay_timer.fetch_sub(1, Ordering::Relaxed);
                }
                if sound_timer.load(Ordering::Relaxed) > 0 {
                    sound_timer.fetch_sub(1, Ordering::Relaxed);
                }

                std::thread::sleep(std::time::Duration::new(0, 16666667));
            }
        }));
    }

    pub fn process_events(&mut self) {
        self.input.process_all_events();
    }

    pub fn should_exit(&mut self) -> bool {
        self.input.should_exit()
    }

    fn decode(&self, instruction: u16) -> Instructions {
        if instruction == 0x00e0 {
            return Instructions::ClearScreen;
        } else if instruction & 0xf000 == 0x1000 {
            return Instructions::UnconditionalJump(instruction & 0x0fff);
        } else if instruction & 0xf000 == 0x6000 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            let intermediate = instruction as u8;
            return Instructions::SetVxToIntermediate(register_identifier, intermediate);
        } else if instruction & 0xf000 == 0x7000 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            let intermediate = instruction as u8;
            return Instructions::AddIntermediateToVx(register_identifier, intermediate);
        } else if instruction & 0xf000 == 0xa000 {
            let address = instruction & 0x0fff;
            return Instructions::SetIndexRegisterToIntermediate(address);
        } else if instruction & 0xf000 == 0xd000 {
            let x_coord = self.data_registers[((instruction & 0x0f00) >> 8) as usize];
            let y_coord = self.data_registers[((instruction & 0x00f0) >> 4) as usize];
            let height = (instruction & 0x000f) as u8;
            return Instructions::DrawSprite(x_coord, y_coord, height);
        } else if instruction & 0xf0ff == 0xe09e {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SkipIfKeyPressedVx(register_identifier);
        } else if instruction & 0xf0ff == 0xe0a1 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SkipIfKeyNotPressedVx(register_identifier);
        } else if instruction & 0xf0ff == 0xf00a {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::AwaitKeyPressVx(register_identifier);
        } else if instruction == 0x00ee {
            return Instructions::ReturnFromSubroutine;
        } else if instruction & 0xf000 == 0x2000 {
            let address = instruction & 0x0fff;
            return Instructions::CallSubroutine(address);
        } else if instruction & 0xf000 == 0x3000 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            let intermediate = instruction as u8;
            return Instructions::SkipIfVxIsIntermediate(register_identifier, intermediate);
        } else if instruction & 0xf000 == 0x4000 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            let intermediate = instruction as u8;
            return Instructions::SkipIfVxIsNotIntermediate(register_identifier, intermediate);
        } else if instruction & 0xf00f == 0x5000 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::SkipIfVxIsVy(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8000 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::SetVxToVy(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8001 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::BitwiseOrVyToVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8002 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::BitwiseAndVyToVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8003 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::BitwiseXorVyToVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8004 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::AddVyToVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8005 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::SubtractVyFromVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x8006 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::StoreLSBfromVxInVf(register_identifier);
        } else if instruction & 0xf00f == 0x800E {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::StoreMSBfromVxInVf(register_identifier);
        } else if instruction & 0xf00f == 0x8007 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::SetVxToVyMinusVx(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf00f == 0x9000 {
            let register_identifier_x = ((instruction & 0x0f00) >> 8) as u8;
            let register_identifier_y = ((instruction & 0x00f0) >> 4) as u8;
            return Instructions::SkipIfVxIsNotVy(register_identifier_x, register_identifier_y);
        } else if instruction & 0xf000 == 0xb000 {
            let address = instruction & 0x0fff;
            return Instructions::UnconditionalJumpWithOffset(address);
        } else if instruction & 0xf000 == 0xc000 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            let intermediate = instruction as u8;
            return Instructions::GenerateRandomNumberWithCap(register_identifier, intermediate);
        } else if instruction & 0xf0ff == 0xf007 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SetVxToDelayTimer(register_identifier);
        } else if instruction & 0xf0ff == 0xf018 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SetSoundTimerToVx(register_identifier);
        } else if instruction & 0xf0ff == 0xf01e {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::AddVxToI(register_identifier);
        } else if instruction & 0xf0ff == 0xf029 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SetIToSpriteLocation(register_identifier);
        } else if instruction & 0xf0ff == 0xf033 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::StoreVxAsBCDInI(register_identifier);
        } else if instruction & 0xf0ff == 0xf055 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::DumpRegisters(register_identifier);
        } else if instruction & 0xf0ff == 0xf065 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::LoadRegisters(register_identifier);
        } else if instruction & 0xf0ff == 0xf015 {
            let register_identifier = ((instruction & 0x0f00) >> 8) as u8;
            return Instructions::SetDelayTimerToVx(register_identifier);
        }

        Instructions::Unkown
    }

    // this is the the whole fetch, decode and execute circle:
    pub fn emulate_cycle(&mut self) {
        let instruction = ((self.memory[self.program_counter as usize] as u16) << 8)
            + (self.memory[(self.program_counter + 1) as usize]) as u16;

        self.program_counter += 2;

        match self.decode(instruction) {
            Instructions::ClearScreen => {
                self.display.clear_screen();
            }
            Instructions::UnconditionalJump(address) => {
                self.program_counter = address;
            }
            Instructions::SetVxToIntermediate(register_identifier, intermediate) => {
                self.data_registers[register_identifier as usize] = intermediate;
            }
            Instructions::AddIntermediateToVx(register_identifier, intermediate) => {
                let (result, _) =
                    self.data_registers[register_identifier as usize].overflowing_add(intermediate);
                self.data_registers[register_identifier as usize] = result;
            }
            Instructions::SetIndexRegisterToIntermediate(address) => {
                self.index_register = address;
            }
            Instructions::DrawSprite(x_coord, y_coord, height) => {
                let was_turned_off = self.display.blend_sprite(
                    x_coord,
                    y_coord,
                    height,
                    self.index_register,
                    &self.memory,
                );

                self.data_registers[0xf] = if was_turned_off { 1 } else { 0 };
            }
            Instructions::SkipIfKeyPressedVx(register_identifier) => {
                if self
                    .input
                    .is_key_pressed(self.data_registers[register_identifier as usize])
                {
                    self.program_counter += 2;
                }
            }
            Instructions::SkipIfKeyNotPressedVx(register_identifier) => {
                if !self
                    .input
                    .is_key_pressed(self.data_registers[register_identifier as usize])
                {
                    self.program_counter += 2;
                }
            }
            Instructions::AwaitKeyPressVx(register_identifier) => {
                // On the original COSMAC VIP, the key was only registered when it was pressed and then released.
                // currently this is not the case: if the key is pressed down, function will return true!
                self.data_registers[register_identifier as usize] = self.input.get_key_blocking();
            }
            Instructions::ReturnFromSubroutine => {
                self.program_counter = self.stack.pop().expect(
                    "ERROR: Tried to pop address from stack, but stack is empty. Exiting...",
                );
            }
            Instructions::CallSubroutine(address) => {
                self.stack.push(self.program_counter);
                self.program_counter = address;
            }
            Instructions::SkipIfVxIsIntermediate(register_identifier, intermediate) => {
                if self.data_registers[register_identifier as usize] == intermediate {
                    self.program_counter += 2;
                }
            }
            Instructions::SkipIfVxIsNotIntermediate(register_identifier, intermediate) => {
                if self.data_registers[register_identifier as usize] != intermediate {
                    self.program_counter += 2;
                }
            }
            Instructions::SkipIfVxIsVy(register_identifier_x, register_identifier_y) => {
                if self.data_registers[register_identifier_x as usize]
                    == self.data_registers[register_identifier_y as usize]
                {
                    self.program_counter += 2;
                }
            }
            Instructions::SetVxToVy(register_identifier_x, register_identifier_y) => {
                self.data_registers[register_identifier_x as usize] =
                    self.data_registers[register_identifier_y as usize];
            }
            Instructions::BitwiseOrVyToVx(register_identifier_x, register_identifier_y) => {
                self.data_registers[register_identifier_x as usize] |=
                    self.data_registers[register_identifier_y as usize];
            }
            Instructions::BitwiseAndVyToVx(register_identifier_x, register_identifier_y) => {
                self.data_registers[register_identifier_x as usize] &=
                    self.data_registers[register_identifier_y as usize];
            }
            Instructions::BitwiseXorVyToVx(register_identifier_x, register_identifier_y) => {
                self.data_registers[register_identifier_x as usize] ^=
                    self.data_registers[register_identifier_y as usize];
            }
            Instructions::AddVyToVx(register_identifier_x, register_identifier_y) => {
                let (result, did_overflow) = self.data_registers[register_identifier_x as usize]
                    .overflowing_add(self.data_registers[register_identifier_y as usize]);
                self.data_registers[register_identifier_x as usize] = result;
                self.data_registers[0xf] = if did_overflow { 1 } else { 0 };
            }
            Instructions::SubtractVyFromVx(register_identifier_x, register_identifier_y) => {
                let (result, did_underflow) = self.data_registers[register_identifier_x as usize]
                    .overflowing_sub(self.data_registers[register_identifier_y as usize]);
                self.data_registers[register_identifier_x as usize] = result;
                self.data_registers[0xf] = if did_underflow { 0 } else { 1 };
            }
            Instructions::StoreLSBfromVxInVf(register_identifier) => {
                self.data_registers[0xf] = self.data_registers[register_identifier as usize] & 0x01;
                self.data_registers[register_identifier as usize] >>= 1;
            }
            Instructions::StoreMSBfromVxInVf(register_identifier) => {
                self.data_registers[0xf] = self.data_registers[register_identifier as usize] & 0x80;
                self.data_registers[register_identifier as usize] <<= 1;
            }
            Instructions::SetVxToVyMinusVx(register_identifier_x, register_identifier_y) => {
                let (result, did_underflow) = self.data_registers[register_identifier_y as usize]
                    .overflowing_sub(self.data_registers[register_identifier_x as usize]);
                self.data_registers[register_identifier_x as usize] = result;
                self.data_registers[0xf] = if did_underflow { 0 } else { 1 };
            }
            Instructions::SkipIfVxIsNotVy(register_identifier_x, register_identifier_y) => {
                if self.data_registers[register_identifier_x as usize]
                    != self.data_registers[register_identifier_y as usize]
                {
                    self.program_counter += 2;
                }
            }
            Instructions::UnconditionalJumpWithOffset(address) => {
                self.program_counter = self.data_registers[0] as u16 + address;
            }
            Instructions::GenerateRandomNumberWithCap(register_identifier, intermediate) => {
                self.data_registers[register_identifier as usize] =
                    rand::random::<u8>() & intermediate;
            }
            Instructions::SetVxToDelayTimer(register_identifier) => {
                self.data_registers[register_identifier as usize] =
                    self.delay_timer.load(Ordering::Relaxed);
            }
            Instructions::SetSoundTimerToVx(register_identifier) => {
                self.sound_timer.store(
                    self.data_registers[register_identifier as usize],
                    Ordering::Relaxed,
                );
            }
            Instructions::AddVxToI(register_identifier) => {
                self.index_register += self.data_registers[register_identifier as usize] as u16;

                if self.index_register >= 0x1000 {
                    self.index_register &= 0x0fff;
                    self.data_registers[0xf] = 1;
                } else {
                    self.data_registers[0xf] = 0;
                }
            }
            Instructions::SetIToSpriteLocation(register_identifier) => {
                self.index_register = FONT_STARTING_MEMORY
                    + self.data_registers[register_identifier as usize] as u16 * 5;
            }
            Instructions::StoreVxAsBCDInI(register_identifier) => {
                let hundreds = self.data_registers[register_identifier as usize] / 100;
                let tens = (self.data_registers[register_identifier as usize] % 100) / 10;
                let ones = self.data_registers[register_identifier as usize] % 10;
                self.memory[self.index_register as usize + 0] = hundreds;
                self.memory[self.index_register as usize + 1] = tens;
                self.memory[self.index_register as usize + 2] = ones;
            }
            Instructions::DumpRegisters(register_identifier) => {
                for reg_offset in 0..register_identifier + 1 {
                    self.memory[self.index_register as usize + reg_offset as usize] =
                        self.data_registers[reg_offset as usize];
                }
            }
            Instructions::LoadRegisters(register_identifier) => {
                for reg_offset in 0..register_identifier + 1 {
                    self.data_registers[reg_offset as usize] =
                        self.memory[self.index_register as usize + reg_offset as usize];
                }
            }
            Instructions::SetDelayTimerToVx(register_identifier) => {
                self.delay_timer.store(
                    self.data_registers[register_identifier as usize],
                    Ordering::Relaxed,
                );
            }
            Instructions::Unkown => {
                panic!(
                    "ERROR: Given instruction: {:#06x} is not known to the emulator.",
                    instruction
                );
            }
        }
    }

    pub fn load_program(&mut self, path: &str) {
        let contents =
            std::fs::read(path).expect("ERROR: Could not load chip8 program. Exiting...");

        for i in 0..contents.len() {
            self.memory[i + PROGRAM_OFFSET as usize] = contents[i];
        }

        // start execution by memory-offset:
        self.program_counter = PROGRAM_OFFSET as u16;
    }
}
