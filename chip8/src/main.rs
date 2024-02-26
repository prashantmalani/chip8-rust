use std::{env, process::exit};

mod mem;
use mem::mem::Memory;

mod cpu;
use cpu::cpu::Cpu;

mod display;
use display::display::Display;

mod timer;
use timer::timer::Timer;

#[show_image::main]
fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() != 2 {
        println!("Invalid number of params:\n Usage is \"cargo run <filepath>\"");
        exit(1);
    }

    // Read file into a vector.
    let program = match std::fs::read(&args[1]) {
        Ok(program) => program,
        Err(_) => {
            println!("Couldn't read file");
            exit(1);
        },
    };

    println!("Read in program of size: {} bytes", program.len()); 

    let mut mem = Memory::new();
    match mem.load_program(&program) {
        Err(e) => println!("Load failed: {}", e),
        _ => {},
    }

    let disp = Display::new(false);

    let mut cpu = Cpu::new();

    let mut timers = Timer::new(false);
    // main loop
    loop {
        let instr = match cpu.fetch(&mem) {
            Ok(instr) => instr,
            Err(e) => {
                println!("Fetch failed: {}", e);
                break;
            },
        };

        match cpu.decode(instr, Some(&disp), Some(&mut mem), Some(&mut timers)) {
            Err(e) => {
                println!("Decode failed: {}", e);
                break;
            },
            _ => {},
        };
    }

    exit(1);
}
