
const MEM_SIZE: usize = 4096;
const PROGRAM_ADDRESS: usize = 0x200;
pub struct Memory {
    pub(crate) mem: [u8; 4096],
}

impl Memory {
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

    pub fn read(&self, addr: usize) -> Result<u8, String> {
        if addr >= MEM_SIZE {
            return Err(String::from("Invalid read address."));
        }

        return Ok(self.mem[addr]);
    }
}

#[cfg(test)]
mod tests {
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
}
