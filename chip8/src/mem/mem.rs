
pub struct Memory {
    pub(crate) mem: [u8; 4096],
}

const PROGRAM_ADDRESS: usize = 0x200;

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
}
