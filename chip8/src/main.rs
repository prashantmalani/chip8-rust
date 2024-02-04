use std::{env, process::exit};

mod mem;
use mem::mem::Memory;

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
}
