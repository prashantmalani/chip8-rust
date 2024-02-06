use std::collections::LinkedList;

use crate::mem::mem::Memory;

pub struct Cpu {
    pc: u16, // program counter
    i: u16, // index register
    v: [u8; 16], // V0-VF
    stack: LinkedList<u16> // Stack
}

const PROGRAM_ADDRESS: u16 = 0x200;

impl Cpu {
    pub fn new() -> Self {
        Cpu {
            pc:  PROGRAM_ADDRESS,
            i: 0x0,
            v: [0; 16],
            stack: LinkedList::new(),

        }
    }

    // Get the next instruction from the PC.
    // Big Endian format.
    pub fn fetch(&mut self, mem: &Memory) -> Result<u16, String> {
        let byte1 = match mem.read(self.pc.into()) {
            Ok(byte) => byte,
            Err(e) => return Err(String::from("Fetch failed") + &e),
        };

        let byte2 = match mem.read((self.pc + 1).into()) {
            Ok(byte) => byte,
            Err(e) => return Err(String::from("Fetch failed") + &e),
        };

        let instruction = ((byte1 as u16) << 8) | byte2 as u16;

        // Increment the PC by 1 instruction immediately.
        self.pc = self.pc + 2;

        return Ok(instruction);
    }

    pub fn decode(&self, instr: u16) -> Result<i32, String>{
        match instr {
            0x00e0 => println!("Received clear screen."),
            unknown_instr => {
                return Err(String::from("Unknown instruction: ") + &unknown_instr.to_string());
            }
        }
        return Ok(0);
    }
}


#[cfg(test)]
mod tests {
    use super::{Memory, Cpu, PROGRAM_ADDRESS};

    #[test]
    // Verify that two consecutive fetches work correctly.
    fn check_fetch() {
        let mut cpu = Cpu::new();
        let mut mem_array: [u8; 4096] = [0; 4096];

        let instr1: u16 = 0x00E0;
        let instr2: u16 = 0x70AB;

        mem_array[PROGRAM_ADDRESS as usize] = ((instr1 >> 8) & 0xFF) as u8;
        mem_array[(PROGRAM_ADDRESS + 1) as usize] = (instr1 & 0xFF) as u8;
        mem_array[(PROGRAM_ADDRESS + 2) as usize] = ((instr2 >> 8) & 0xFF) as u8;
        mem_array[(PROGRAM_ADDRESS + 3) as usize] = (instr2 & 0xFF) as u8;

        let mem = Memory {
            mem: mem_array,
        };

        assert_eq!(cpu.fetch(&mem).unwrap(), instr1);
        assert_eq!(cpu.pc, PROGRAM_ADDRESS + 2);
        assert_eq!(cpu.fetch(&mem).unwrap(), instr2);
    }

    #[test]
    fn check_invalid_addr() {
        let mut cpu = Cpu::new();
        let mem = Memory {
            mem: [0; 4096],
        };
    
        cpu.pc = 4096 + 10;
        assert!(cpu.fetch(&mem).is_err());
    }

    #[test]
    fn check_decode() {
        let cpu = Cpu::new();
        assert!(cpu.decode(0x00e0).is_ok());
        assert!(cpu.decode(0x9000).is_err());
    }
}