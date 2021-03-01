use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

// XXX assignment of numerical values to opcodes matters for the decoder and is still TBD
#[derive(FromPrimitive)]
pub enum Anti80Opcode {
    Imm10,
    Jal,
    Skipc,
    Sw,
    Sb,
    Lw,
    Lb,
    Lbu,
    Mov,
    Add,
    Subr,
    And,
    Or,
    Xor,
    Sr,
    Sl,
}

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
        use Anti80Opcode::*;

        let imm = 0;
        let src1 = if true { self.reg[0] } else { imm };

        let result = match FromPrimitive::from_u16(opcode).expect("can't happen") {
            Imm10 => panic!("Haven't implemented this instruction yet"),
            Jal => panic!("Haven't implemented this instruction yet"),
            Skipc => panic!("Haven't implemented this instruction yet"),
            Sw => panic!("Haven't implemented this instruction yet"),
            Sb => panic!("Haven't implemented this instruction yet"),
            Lw => panic!("Haven't implemented this instruction yet"),
            Lb => panic!("Haven't implemented this instruction yet"),
            Lbu => panic!("Haven't implemented this instruction yet"),
            Mov => src1,
            Add => panic!("Haven't implemented this instruction yet"),
            Subr => panic!("Haven't implemented this instruction yet"),
            And => panic!("Haven't implemented this instruction yet"),
            Or => panic!("Haven't implemented this instruction yet"),
            Xor => panic!("Haven't implemented this instruction yet"),
            Sr => panic!("Haven't implemented this instruction yet"),
            Sl => panic!("Haven't implemented this instruction yet"),
        };

        if false {
            self.reg[0] = result;
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
}
