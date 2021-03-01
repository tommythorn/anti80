#[derive(Debug)]
// XXX should probably keep memory outside the processor state
pub struct Anti80 {
    pub memory: Vec<u8>,
    pub pc: u16,
    pub reg: Vec<u16>,
}

impl Anti80 {
    pub fn new() -> Anti80 {
        Self {
            memory: vec![0; 65536],
            pc: 0,
            reg: vec![0, 8],
        }
    }

    pub fn load16(&self, addr: u16) -> u16 {
        assert!((addr & 1) == 0);
        let lo: u16 = self.memory[addr as usize].into();
        let hi: u16 = self.memory[addr as usize + 1].into();
        hi * 256 + lo
    }

    pub fn step(&mut self) {
        // fetch
        let insn = self.load16(self.pc);
        let opcode = insn >> 12;
        match opcode {
            _ => panic!("Haven't implemented opcode {} yet", opcode),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn init_and_step() {
        let mut anti80 = Anti80::new();
        anti80.step();
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
