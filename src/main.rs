use std::thread::sleep;
use std::time::{Duration, Instant};

use clap::{App, Arg};

// TODO: fix timing-stuff in main: both emulated cycles and screen refreshrates
// TODO: are not where they should be! (maybe completely different approach?)

mod chip8;

const DEFAULT_FREQUENCY: &str = "700";

fn parse_command_arguments() -> (String, u32) {
    let matches = App::new("CHIP-8 Emulator")
        .version("0.0.1")
        .author("Felix Röpke")
        .about("A Simple CHIP-8 emulator written in Rust")
        .arg(
            Arg::with_name("path")
                .long("path")
                .short("p")
                .help("Path to a valid CHIP-8 ROM")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("frequency")
                .long("frequency")
                .short("f")
                .help("The number of CHIP-8 instructions per second")
                .takes_value(true),
        )
        .get_matches();

    let path = matches
        .value_of("path")
        .expect("ERROR: No ROM given as an argument. Exiting...")
        .to_string();
    let frequency = matches.value_of("frequency").unwrap_or(DEFAULT_FREQUENCY);
    let frequency = frequency
        .parse::<u32>()
        .expect("ERROR: Could not parse given frequency to integer. Exiting...");
    (path, frequency)
}

fn main() {
    let (path, frequency) = parse_command_arguments();

    let mut chip8 = chip8::Chip8::init();

    chip8.load_program(&path);
    chip8.start_timers();
    chip8.start_sound_system();

    let mut display_time = Instant::now();

    let mut count_display_time = Instant::now();
    let mut cycle_count = 0;
    let mut draw_count = 0;

    while !chip8.should_exit() {
        // emulate cycle
        let time_before = Instant::now();
        chip8.process_events();
        chip8.emulate_cycle();
        cycle_count += 1;
        let cycle_time = Instant::now().duration_since(time_before);

        // at 60Hz, update the screen
        let crnt_time = Instant::now();
        if crnt_time - display_time >= Duration::new(0, 16666667) {
            chip8.draw_display();
            display_time = crnt_time;
            draw_count += 1;
        }

        if crnt_time - count_display_time >= Duration::new(1, 0) {
            println!("Cycles in this second: {}", cycle_count);
            println!("Draws in this second: {}", draw_count);
            cycle_count = 0;
            draw_count = 0;
            count_display_time = crnt_time;
        }

        // sleep for rest of the duration until next cycle
        let sleep_per_cycle = Duration::new(0, 1_000_000_000 / frequency);
        sleep_per_cycle.checked_sub(cycle_time).take().map(sleep);
    }

    // TODO: dont forget to implement drop for chip8: you must de-init everything
    // TODO: (do it recursively for display-sdl2 etc.) and join the threads!
    println!("Stopping CHIP-8's timer-thread...");
    chip8.stop_sound_system();
    chip8.stop_timers();
}
