use std::{env, process::exit};

mod mem;
use mem::mem::Memory;

mod cpu;
use cpu::cpu::Cpu;

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

    let mut mem = Memory{mem: [0; 4096]}; 
    match mem.load_program(&program) {
        Err(e) => println!("Load failed: {}", e),
        _ => {},
    }

    let mut cpu = Cpu::new();
    // main loop
    loop {
        let instr = match cpu.fetch(&mem) {
            Ok(instr) => instr,
            Err(e) => {
                println!("Fetch failed: {}", e);
                break;
            },
        };

        let decode = match cpu.decode(instr) {
            Err(e) => {
                println!("Decode failed: {}", e);
                break;
            },
            _ => {},
        };
    }

    exit(1);
}
