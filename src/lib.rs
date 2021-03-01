use modular_bitfield::prelude::*;
// XXX assignment of numerical values to opcodes matters for the decoder and is still TBD
#[derive(BitfieldSpecifier, PartialEq, Eq)]
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

#[bitfield]
pub struct Anti80Insn {
    src2: B5,
    src1: B3,
    dest: B3,
    sign: B1,
    opcode: Anti80Opcode,
}

#[derive(Debug)]
// XXX should probably keep memory outside the processor state
pub struct Anti80 {
    pub memory: Vec<u8>,
    pub pc: i16,
    pub reg: Vec<i16>,
}

impl Anti80 {
    pub fn new() -> Anti80 {
        Self {
            memory: vec![0; 65536],
            pc: 0,
            reg: vec![0; 8],
        }
    }

    pub fn load8(&self, addr: i16) -> i16 {
        self.memory[addr as usize] as i16
    }

    pub fn store8(&mut self, addr: i16, data: i16) {
        self.memory[addr as usize] = data as u8
    }

    pub fn load16(&self, addr: i16) -> [u8; 2] {
        assert!((addr & 1) == 0);
        [self.memory[addr as usize], self.memory[addr as usize + 1]]
    }

    pub fn store_insn(&mut self, addr: i16, insn: Anti80Insn) {
        let bytes = insn.into_bytes();
        self.memory[addr as usize] = bytes[0];
        self.memory[addr as usize + 1] = bytes[1];
    }

    fn asm(&mut self, addr: &mut i16, op: Anti80Opcode, sign: u8, dest: u8, src1: u8, src2: u8) {
        self.store_insn(
            *addr,
            Anti80Insn::new()
                .with_opcode(op)
                .with_sign(sign)
                .with_dest(dest)
                .with_src1(src1)
                .with_src2(src2),
        );
        *addr += 2;
    }

    pub fn step(&mut self) {
        // fetch
        let insn = Anti80Insn::from_bytes(self.load16(self.pc));
        self.pc += 2;
        use Anti80Opcode::*;

        let src1 = insn.src1() as i16;
        let src2 = insn.src2() as i16;
        let dest = insn.dest() as i16;
        let rd = dest as usize;
        let sign = insn.sign() != 0;
        let opcode = insn.opcode();

        let store_simm6 = {
            let uimm5 = ((src2 >> 3) & 0x18) + dest;
            if sign {
                uimm5 - 32
            } else {
                uimm5
            }
        };
        let jal_simm12 = {
            let uimm11 = (src2 >> 3 << 9) + (dest << 6) + (src1 << 3) + (src2 & 7);
            if sign {
                uimm11 - 2048
            } else {
                uimm11
            }
        };
        let simm6 = if sign { src2 - 32 } else { src2 };

        let src2_is_reg = opcode == Sw || opcode == Sb || sign && src2 >= 0x20;
        let rs1 = self.reg[src1 as usize] as i16;
        let rs2 = self.reg[src2 as usize & 7] as i16;
        let rs2_imm = if opcode == Sw || opcode == Sb {
            store_simm6
        } else if src2_is_reg {
            rs2
        } else {
            simm6
        };

        match opcode {
            Imm10 => panic!("Haven't implemented this instruction yet"),
            Jal => {
                self.reg[7] = self.pc;
                self.pc += (jal_simm12 << 1) as i16;
            }
            Skipc => panic!("Haven't implemented this instruction yet"),
            Sw => self.store8(rs2_imm + rs1, rs2),
            Sb => {
                self.store8(rs2_imm + rs1, rs2);
                self.store8(rs2_imm + rs1 + 1, rs2 >> 8);
            }
            Lw => {
                let lo = self.load8(rs2_imm + rs1);
                let hi = self.load8(rs2_imm + rs1 + 1);
                self.reg[rd] = hi * 256 + lo;
            }
            Lb => {
                let b = self.load8(rs2_imm + rs1);
                self.reg[rd] = if b < 128 { b } else { b - 256 }
            }
            Lbu => self.reg[rd] = self.load8(rs2_imm + rs1),
            Mov => self.reg[rd] = rs2_imm,
            Add => self.reg[rd] = rs2_imm + rs1,
            Subr => self.reg[rd] = rs2_imm - rs1,
            And => self.reg[rd] = rs2_imm & rs1,
            Or => self.reg[rd] = rs2_imm | rs1,
            Xor => self.reg[rd] = rs2_imm ^ rs1,
            Sr => {
                self.reg[rd] = if (src2 & 16) != 0 {
                    rs1 >> rs2_imm
                } else {
                    (rs1 as u16 >> rs2_imm) as i16
                }
            }
            Sl => self.reg[rd] = rs1 << rs2_imm,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_and_step() {
        use Anti80Opcode::*;
        let mut addr: i16 = 0;
        let mut anti80 = Anti80::new();
        anti80.asm(&mut addr, Mov, 0, 5, 0, 14);
        anti80.asm(&mut addr, Mov, 0, 6, 0, 7);
        anti80.asm(&mut addr, Add, 1, 7, 5, 6 | 0x18); // XXX <<< not picking up on register
        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 0, 0]);
        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 7, 0]);
        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 7, 21]);
    }
}
