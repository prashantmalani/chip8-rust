use std::collections::LinkedList;

use crate::{mem::mem::Memory, display::display::{Display, WIDTH, HEIGHT}};

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

    fn set_v(&mut self, instr: u16) {
        let ind = (instr >> 8) & 0xF;
        let val = (instr & 0xFF) as u8;

        self.v[ind as usize] = val;
    }

    fn add_v(&mut self, instr: u16) {
        let val = instr & 0xFF;
        let ind = (instr >> 8) & 0xF;

        let old_reg = self.v[ind as usize] as u32;
        let new_reg = (old_reg + val as u32) as u8;
        // Don't update the VF register even if there is an overflow.
        self.v[ind as usize] = new_reg;
    }

    fn handle_jump(&mut self, instr: u16) {
        self.pc = instr & 0xFFF;
    }

    fn skip_vx_equal(&mut self, instr: u16) {
        let val = instr & 0xFF;
        let x = (instr >> 8) & 0xF;

        if self.v[x as usize] == val as u8 {
            self.pc = self.pc + 2;
        }
    }

    fn skip_vx_ne(&mut self, instr: u16) {
        let val = instr & 0xFF;
        let x = (instr >> 8) & 0xF;

        if self.v[x as usize] != val as u8 {
            self.pc = self.pc + 2;
        }
    }

    fn skip_vx_vy_equal(&mut self, instr: u16) {
        let x = (instr >> 8) & 0xF;
        let y = (instr >> 4) & 0xF;

        if self.v[x as usize] == self.v[y as usize] {
            self.pc = self.pc + 2;
        }
    }

    fn arith_vx_minus_vy(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        let vx = self.v[x_ind as usize];
        let vy = self.v[y_ind as usize];
        let mut result: u8;
        if vx > vy {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        let result = vx.wrapping_sub(vy);
        self.v[x_ind as usize] = result;
    }

    fn arith_vy_minus_vx(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        let vx = self.v[x_ind as usize];
        let vy = self.v[y_ind as usize];
        let mut result: u8;
        if vy > vx {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        let result = vy.wrapping_sub(vx);
        self.v[x_ind as usize] = result;
    }

    fn logic_vx_or_vy(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        let vx = self.v[x_ind as usize];
        let vy = self.v[y_ind as usize];

        self.v[x_ind as usize] = vx | vy;
    }

    fn logic_vx_and_vy(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        let vx = self.v[x_ind as usize];
        let vy = self.v[y_ind as usize];

        self.v[x_ind as usize] = vx & vy;
    }

    fn logic_vx_xor_vy(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        let vx = self.v[x_ind as usize];
        let vy = self.v[y_ind as usize];

        self.v[x_ind as usize] = vx ^ vy;
    }

    fn left_shift(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        // TODO: Add a config to control this behavior
        //self.v[x_ind as usize] = self.v[y_ind as usize];
        let vx = self.v[x_ind as usize];

        if (vx & 0x80) >> 0x7 == 1 {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x_ind as usize] = vx << 1;
    }

    fn right_shift(&mut self, instr: u16) {
        let x_ind = (instr >> 8) & 0xF;
        let y_ind = (instr >> 4) & 0xF;

        // TODO: Add a config to control this behavior
        //self.v[x_ind as usize] = self.v[y_ind as usize];
        let vx = self.v[x_ind as usize];

        if (vx & 0x1) == 1 {
            self.v[0xF] = 1;
        } else {
            self.v[0xF] = 0;
        }

        self.v[x_ind as usize] = vx >> 1;
    }

    fn handle_logic_arith(&mut self, instr: u16) -> Result<i32, String> {
        match instr & 0xF {
            5 => self.arith_vx_minus_vy(instr),
            6 => self.right_shift(instr),
            7 => self.arith_vy_minus_vx(instr),
            1 => self.logic_vx_or_vy(instr),
            2 => self.logic_vx_and_vy(instr),
            3 => self.logic_vx_xor_vy(instr),
            0xE => self.left_shift(instr),
            _ => return Err(String::from("Unhandled instruction: 0x") + format!("{:X}", &instr).as_str()),
        }
        return Ok(0);
    }

    fn get_font_char(&self, instr: u16) -> u8 {
        let x_ind = (instr >> 8) & 0xF;
        return self.v[x_ind as usize] & 0xF;
    }

    fn font_character(&mut self, instr: u16, mem: &Memory) {
        let chr = self.get_font_char(instr);
        self.i = mem.get_font_addr(chr) as u16;
    }

    fn store(&self, instr: u16, mem: &mut Memory) {
        // TODO: Add config to update the i with each copy.
        let ind = (instr >> 8)  & 0xF;
        for i in 0..=ind {
            mem.mem[(self.i + i) as usize] = self.v[i as usize];
        }
    }

    fn load(&mut self, instr: u16, mem: &Memory) {
        // TODO: Add config to update the i with each copy.
        let ind = (instr >> 8)  & 0xF;
        for i in 0..=ind {
            self.v[i as usize] = mem.mem[(self.i + i) as usize];
        }
    }

    fn bcd(&self, instr: u16, mem: &mut Memory) {
        let x = (instr >> 8) & 0xF;
        let mut val = self.v[x as usize];

        let digit3 = val % 10;
        val = val / 10;
        let digit2 = val % 10;
        val = val / 10;
        let digit1 = val % 10;

        mem.mem[self.i as usize] = digit1;
        mem.mem[(self.i + 1) as usize] = digit2;
        mem.mem[(self.i + 2) as usize] = digit3;
    }

    fn increment_i(&mut self, instr: u16) {
        let x = (instr >> 8) & 0xF;
        let val = self.v[x as usize];

        let old_i = self.i as u32;
        let result = old_i + val as u32;
        if result >= 4096 {
            self.v[0xF] = 1
        }
        self.i = (result & 0xFFFF) as u16;
    }

    fn handle_f_instructions(&mut self, instr: u16, mem: Option<&mut Memory>) -> Result<i32, String> {
        match instr & 0xFF {
            0x1E => self.increment_i(instr),
            0x29 => self.font_character(instr, &*mem.unwrap()),
            0x33 => self.bcd(instr, mem.unwrap()),
            0x55 => self.store(instr, mem.unwrap()),
            0x65 => self.load(instr, mem.unwrap()),
            _ => return Err(String::from("Unhandled instruction: 0x")  + format!("{:X}", &instr).as_str())
        }
        return Ok(0);
    }

    /*
       Decodes the draw instruction: DXYN

       In order to make the code more testable, we split it into two parts:
       1. Gets all the sprite data from the index, along with X,Y coordinates.
       2. Draw command which basically calls the displays draw command

       This way, we can unit test the CPU section of the logic (part 1) while
       the display module can effectively unit test the display logic (part 2)
       of the code.
    */
    fn get_sprite(&self, instr: u16, mem: &Memory) -> (u8, u8, Vec<u8>) {
        let x_reg_ind = ((instr >> 8) & 0xF) as usize;
        let y_reg_ind = ((instr >> 4) & 0xF) as usize;

        let x = self.v[x_reg_ind] % (WIDTH as u8);
        let y = self.v[y_reg_ind] % (HEIGHT as u8);
        let n = instr & 0xF;

        let mut sprite: Vec<u8> = Vec::new();
        for ind in 0..n {
            sprite.push(mem.mem[self.i as usize + ind as usize])
        }

        return (x, y, sprite);
    }

    fn handle_draw(&mut self, instr: u16, mem: Option<&Memory>, disp: &mut Display) {
        let (x, y, sprite) =self.get_sprite(instr, mem.unwrap());
        self.v[0xf] = disp.draw(x, y, &sprite);
    }

    pub fn decode(&mut self, instr: u16, disp: Option<&mut Display>, mem: Option<&mut Memory>) -> Result<i32, String>{
            match instr {
            0x00e0 => disp.unwrap().clear(),
            instr2 => {
                match (instr2 >> 12) & 0xF {
                    0x1 => self.handle_jump(instr2),
                    0x3 => self.skip_vx_equal(instr2),
                    0x4 => self.skip_vx_ne(instr2),
                    0x5 => self.skip_vx_vy_equal(instr2),
                    0xA => self.set_i(instr2),
                    0x6 => self.set_v(instr2),
                    0x7 => self.add_v(instr2),
                    0x8 => if let Err(e) = self.handle_logic_arith(instr2) {
                        return Err(e);
                    },
                    0xD => self.handle_draw(instr2, Some(&*mem.unwrap()), &mut disp.unwrap()),
                    0xF => if let Err(e) = self.handle_f_instructions(instr2, mem) {
                        return Err(e);
                    }
                    _ => {
                        return Err(String::from("Unknown instruction: 0x") + format!("{:X}", &instr2).as_str());
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
        assert!(cpu.decode(0x9000, None, None).is_err());
    }

    #[test]
    fn decode_disp_clear() {
        let mut cpu = Cpu::new();
        let mut disp = Display::new();
        assert!(cpu.decode(0x00e0, Some(&mut disp), None).is_ok());
    }

    #[test]
    fn decode_set_i() {
        let mut cpu = Cpu::new();
        assert!(cpu.decode(0xa22a, None, None).is_ok());
        assert_eq!(cpu.i, 0x22a);
    }

    #[test]
    fn decode_set_v() {
        let mut cpu = Cpu::new();
        assert!(cpu.decode(0x600c, None, None).is_ok());
        assert_eq!(cpu.v[0], 0xc);
        assert!(cpu.decode(0x6FFE, None, None).is_ok());
        assert_eq!(cpu.v[0xF], 0xFE);  
    }

    #[test]
    fn decode_add_v() {
        let mut cpu = Cpu::new();
        let x = 0x4 as usize;
        let nn = 0x32;
        cpu.v[x] = 0x32;
        let instr = ((0x7 << 12) | (x << 8) | nn) as u16;
        cpu.add_v(instr);
        assert_eq!(cpu.v[x], 0x64);
        assert_eq!(cpu.v[0xf], 0);

        // Test the overflow case.
        cpu.v[x] = 0xFF;
        cpu.add_v(instr);
        assert_eq!(cpu.v[x], 0x31);
        assert_eq!(cpu.v[0xf], 0);
    }

    #[test]
    fn handle_jump() {
        let mut cpu = Cpu::new();
        let instr = (0x1 << 12) | 0x123;

        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, 0x123);
    }

    #[test]
    fn decode_skip_vx_eq() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const NN: u8 = 0x45;
        let instr = ((0x3 << 12) | (X as u16 )<< 8 | NN as u16) as u16;
        const ORIG_PC: u16 = 0x500;
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = NN;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC + 2);

        // Now change the VX value, so we can check the not-equal case.
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = NN + 1;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC);
    }

    #[test]
    fn decode_skip_vx_ne() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const NN: u8 = 0x45;
        let instr = ((0x4 << 12) | (X as u16 )<< 8 | NN as u16) as u16;
        const ORIG_PC: u16 = 0x500;
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = NN;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC);

        // Now change the VX value, so we can check the not-equal case.
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = NN + 1;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC + 2);
    }

    #[test]
    fn decode_skip_vx_vy_eq() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL: u8 = 0x45;
        let instr = ((0x5 << 12) | (X as u16 ) << 8 | (Y as u16) << 4);
        const ORIG_PC: u16 = 0x500;
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = VAL;
        cpu.v[Y as usize] = VAL;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC + 2);

        // Now change the VX value, so we can check the not-equal case.
        cpu.pc = ORIG_PC;
        cpu.v[X as usize] = VAL + 1;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.pc, ORIG_PC);
    }

    #[test]
    fn decode_arith_vx_minus_vy() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0x50;
        const VAL2: u8 = 0x45;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x5;

        cpu.v[X as usize] = VAL1 as u8;
        cpu.v[Y as usize] = VAL2 as u8;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], VAL1 - VAL2);
        assert_eq!(cpu.v[0xF], 1);

        // Swap the values so we can see how the underflow works.
        cpu.v[X as usize] = VAL2 as u8;
        cpu.v[Y as usize] = VAL1 as u8;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], VAL2.wrapping_sub(VAL1));
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn decode_arith_vy_minus_vx() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0x50;
        const VAL2: u8 = 0x45;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x7;

        cpu.v[Y as usize] = VAL1 as u8;
        cpu.v[X as usize] = VAL2 as u8;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], VAL1 - VAL2);
        assert_eq!(cpu.v[0xF], 1);

        // Swap the values so we can see how the underflow works.
        cpu.v[Y as usize] = VAL2 as u8;
        cpu.v[X as usize] = VAL1 as u8;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], VAL2.wrapping_sub(VAL1));
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn decode_logic_vx_or_vy() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0xF;
        const VAL2: u8 = 0xF0;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x1;

        cpu.v[X as usize] = VAL1;
        cpu.v[Y as usize] = VAL2;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0xFF);
    }

    #[test]
    fn decode_logic_vx_and_vy() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0xFF;
        const VAL2: u8 = 0x3;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x2;

        cpu.v[X as usize] = VAL1;
        cpu.v[Y as usize] = VAL2;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0x3);
    }

    #[test]
    fn decode_logic_vx_xor_vy() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0xAA;
        const VAL2: u8 = 0x55;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x3;

        cpu.v[X as usize] = VAL1;
        cpu.v[Y as usize] = VAL2;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0xFF);
    }

    #[test]
    fn decode_left_shift() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0xAA;
        const VAL2: u8 = 0x55;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0xE;

        cpu.v[X as usize] = VAL1;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0x54);
        assert_eq!(cpu.v[0xF], 1);

        cpu.v[X as usize] = VAL2;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0xAA);
        assert_eq!(cpu.v[0xF], 0);
    }

    #[test]
    fn decode_right_shift() {
        let mut cpu = Cpu::new();
        const X: u8 = 0x2;
        const Y: u8 = 0x3;
        const VAL1: u8 = 0xAA;
        const VAL2: u8 = 0x55;
        let instr = ((0x8 << 12) | (X as u16 ) << 8 | (Y as u16) << 4) | 0x6;

        cpu.v[X as usize] = VAL1;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0x55);
        assert_eq!(cpu.v[0xF], 0);

        cpu.v[X as usize] = VAL2;
        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.v[X as usize], 0x2A);
        assert_eq!(cpu.v[0xF], 1);
    }

    // The memory fetch aspect is tested in the memory module, so we just need to test that
    // we can get the character value out correctly.
    #[test]
    fn get_font_char() {
        let mut cpu = Cpu::new();
        const X: usize = 0x4;
        cpu.v[X] = 0xA;
        let instr = 0xF << 12 | (X << 8)  as u16 | 0x29;
        assert_eq!(cpu.get_font_char(instr), 0xA)
    }

    #[test]
    fn store() {
        let mut cpu = Cpu::new();
        let mut mem = Memory { mem: [0; 4096] };
        const I : usize = 0x600;
        const X: u8 = 0x4;
        const VAL: u8 = 0xAA;
        let instr = (0xF << 12) | (X as u16) << 8 | 0x55;

        for i in 0..=X {
            cpu.v[i as usize] = VAL;
        }
        cpu.i = I as u16;

        assert!(cpu.decode(instr, None, Some(&mut mem)).is_ok());
        for j in 0..=X {
            assert_eq!(mem.mem[I + j as usize], VAL);
        }

        // Make sure that the next memory address was unaffected.
        assert_eq!(mem.mem[I + 1+ X as usize], 0);
    }

    #[test]
    fn load() {
        let mut cpu = Cpu::new();
        let mut mem = Memory { mem: [0; 4096] };
        const I : usize = 0x600;
        const X: u8 = 0x4;
        const VAL: u8 = 0xAA;
        let instr = (0xF << 12) | (X as u16) << 8 | 0x65;

        // Load up the memory.
        for i in 0..16 {
            mem.mem[I + i as usize] = VAL
        }
        cpu.i = I as u16;

        assert!(cpu.decode(instr, None, Some(&mut mem)).is_ok());
        for j in 0..=X {
            assert_eq!(cpu.v[j as usize], VAL);
        }

        // Make sure that the next memory address was unaffected.
        assert_eq!(cpu.v[X as usize + 1], 0);
    }

    #[test]
    fn bcd() {
        let mut cpu = Cpu::new();
        let mut mem = Memory { mem: [0; 4096]};
        const I: usize = 0x500;
        const X: u8 = 0x4;
        const VAL: u8 = 139;

        let instr = (0xF << 12) | (X as u16) << 8 | 0x33;
        cpu.i = I as u16;
        cpu.v[X as usize] = VAL;

        assert!(cpu.decode(instr, None, Some(&mut mem)).is_ok());

        assert_eq!(mem.mem[I], 1);
        assert_eq!(mem.mem[I + 1], 3);
        assert_eq!(mem.mem[I + 2], 9);
    }

    #[test]
    fn increment_i() {
        let mut cpu = Cpu::new();
        let mut mem = Memory { mem: [0; 4096] } ;

        const I: usize = 0x500;
        const X: u8 = 0x4;
        const VAL: u8 = 0x24;

        let instr = (0xF << 12) | (X as u16) << 8 | 0x1E;
        cpu.i = I as u16;
        cpu.v[X as usize] = VAL;

        assert!(cpu.decode(instr, None, None).is_ok());
        assert_eq!(cpu.i, (I + VAL as usize) as u16);
    }

    #[test]
    fn get_sprite() {
        let mut cpu = Cpu::new();
        // TODO: Find a way to use MEM_SIZE constant.
        let mut mem_buf = [0; 4096];

        // Fill up a buffer with a sprite:
        const I: u16 = 0x400;
        const N: u8 = 5;
        let expected_sprite: [u8; N as usize] = [0x34, 0x88, 0x88, 0x23, 0x01];
        for i in 0..N {
            mem_buf[I as usize + i as usize] = expected_sprite[i as usize];
        }

        let memory = Memory { mem: mem_buf };

        // Set up CPU registers
        let x = 34;
        let y = 12;
        let x_reg = 4;
        let y_reg = 8;
        cpu.v[x_reg] = x;
        cpu.v[y_reg] = y;
        cpu.i = I;

        let instr: u16 = (N as u16) | (y_reg << 4) as u16 | (x_reg << 8) as u16 | (0xD << 12) as u16;
        let (ret_x,ret_y, vec) = cpu.get_sprite(instr, &memory);
        assert_eq!(ret_x, x);
        assert_eq!(ret_y, y);
        assert_eq!(&vec[..], &expected_sprite[..]);
    }

}