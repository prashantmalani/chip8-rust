
const MEM_SIZE: usize = 4096;
const PROGRAM_ADDRESS: usize = 0x200;
const FONT_ADDRESS: usize = 0x50;
const FONT_HEIGHT: usize = 5;

pub struct Memory {
    pub(crate) mem: [u8; 4096],
}

impl Memory {
    pub fn new() -> Self {
       let mut mem = Memory { mem: [0; 4096]};
       mem.load_font();
       return mem;
    }

    // Program's are stored at 0x200 onwards
    pub fn load_program(&mut self, program: &Vec<u8>) -> Result<i32, String> {
        if program.len() > (self.mem.len() - PROGRAM_ADDRESS) {
            return Err(String::from("Program is too large."));
        }

        let mut i = PROGRAM_ADDRESS;
        for byte in program.iter() {
            self.mem[i] = *byte;
            i = i + 1;
        }

        return Ok(0);
    }

    // Load system font into the memory.
    fn load_font(&mut self) {
        const FONT_ARRAY: [u8; 80] = [
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
            0xF0, 0x80, 0xF0, 0x80, 0x80  // F
        ];

        for (i, val) in FONT_ARRAY.iter().enumerate() {
            self.mem[FONT_ADDRESS + i] = *val;
        }
    }

    pub fn get_font_addr(&self, font: u8) -> usize {
        return FONT_ADDRESS + (FONT_HEIGHT * (font & 0xF) as usize);
    }

    pub fn read(&self, addr: usize) -> Result<u8, String> {
        if addr >= MEM_SIZE {
            return Err(String::from("Invalid read address."));
        }

        return Ok(self.mem[addr]);
    }
}

#[cfg(test)]
mod tests {
    use crate::mem::mem::{FONT_ADDRESS, FONT_HEIGHT};

    use super::Memory;

    #[test]
    fn check_invalid_size() {
        let large_program = vec![0; 4000];
        let mut mem = Memory{mem: [0; 4096]}; 
        assert!(mem.load_program(&large_program).is_err());
    }

    #[test]
    fn check_load() {
        let prog: Vec<u8> = vec![0x8; 400];
        let mut mem = Memory{mem: [0; 4096]};
        assert!(mem.load_program(&prog).is_ok());

        assert_eq!(mem.read(crate::mem::mem::PROGRAM_ADDRESS).unwrap(), 0x8);
        assert_eq!(mem.read(crate::mem::mem::PROGRAM_ADDRESS + prog.len() - 1).unwrap(), 0x8);
    }

    #[test]
    fn get_font_addr() {
        let mem = Memory::new();
        assert_eq!(mem.get_font_addr(0x4), FONT_ADDRESS + (0x4 * FONT_HEIGHT));
    }
}
