use std::collections::LinkedList;

use crate::{mem::mem::Memory, display::display::Display};

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

    // Handler for the "Set I" instruction.
    fn set_i(&mut self, instr: u16) {
        self.i = instr & 0xFFF;
    }

    pub fn decode(&mut self, instr: u16, disp: &mut Display) -> Result<i32, String>{
        match instr {
            0x00e0 => disp.clear(),
            instr2 => {
                match (instr2 >> 12) & 0xF {
                    0xA => self.set_i(instr2),
                    _ => {
                        return Err(String::from("Unknown instruction: ") + &instr2.to_string());
                    }
                }
            }


        }
        return Ok(0);
    }
}


#[cfg(test)]
mod tests {
    use crate::display::display::Display;

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
    fn fetch_invalid_addr() {
        let mut cpu = Cpu::new();
        let mem = Memory {
            mem: [0; 4096],
        };
    
        cpu.pc = 4096 + 10;
        assert!(cpu.fetch(&mem).is_err());
    }

    #[test]
    fn decode_invalid() {
        let mut cpu = Cpu::new();
        let mut disp = Display::new();
        assert!(cpu.decode(0x9000, &mut disp).is_err());
    }

    #[test]
    fn decode_disp_clear() {
        let mut cpu = Cpu::new();
        let mut disp = Display::new();
        assert!(cpu.decode(0x00e0, &mut disp).is_ok());
    }

    #[test]
    fn decode_set_i() {
        let mut cpu = Cpu::new();
        let mut disp = Display::new();
        assert!(cpu.decode(0xa22a, &mut disp).is_ok());
        assert_eq!(cpu.i, 0x22a);
    }

}