use modular_bitfield::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
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
use Anti80Opcode::*;

#[derive(FromPrimitive)]
pub enum Anti80Reg {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}
use Anti80Reg::*;

#[bitfield]
pub struct Anti80Insn {
    src2: B5,
    src1: B3,
    dest: B3,
    sign: B1,
    opcode: Anti80Opcode,
}
#[derive(FromPrimitive)]
pub enum Anti80SkipCond {
    Beq,
    Bne,
    Uimp2,
    Uimp3,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

#[derive(Debug)]
pub struct Anti80 {
    pub memory: Vec<u8>, // XXX should probably keep memory outside the processor state
    pub pc: i16,
    pub reg: Vec<i16>,
    pub asm_addr: i16, // Just a convenience to make the code more readable
}

impl Anti80 {
    pub fn new() -> Anti80 {
        Self {
            memory: vec![0; 65536],
            pc: 0,
            reg: vec![0; 8],
            asm_addr: 0,
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
    fn asm_internal(&mut self, op: Anti80Opcode, sign: u8, dest: u8, src1: u8, src2: u8) {
        self.store_insn(
            self.asm_addr,
            Anti80Insn::new()
                .with_opcode(op)
                .with_sign(sign)
                .with_dest(dest)
                .with_src1(src1)
                .with_src2(src2),
        );
        self.asm_addr += 2;
    }
    pub fn asm_alu(&mut self, op: Anti80Opcode, dest: Anti80Reg, rs1: Anti80Reg, rs2: Anti80Reg) {
        self.asm_internal(op, 1, dest as u8, rs1 as u8, (rs2 as u8) | 0x18)
    }
    fn asm_alui(&mut self, op: Anti80Opcode, dest: Anti80Reg, rs1: Anti80Reg, imm: i16) {
        assert!(-24 <= imm && imm < 32); // XXX Don't handle IMM10 yet
        let sign = if imm < 0 { 1 } else { 0 };
        self.asm_internal(op, sign, dest as u8, rs1 as u8, imm as u8)
    }

    pub fn asm_jal(&mut self, rs2: Anti80Reg) {
        self.asm_internal(Jal, 1, 0, 0, 0x18 + (rs2 as u8));
    }
    pub fn asm_jali(&mut self, target: i16) {
        let mut delta = target.wrapping_sub(self.asm_addr);
        assert!((delta & 1) == 0);
        delta = delta / 2;
        assert!(-2048 <= delta && delta < 2047); // XXX No IMM10 support yet
        let imm86 = ((delta >> 6) & 7) as u8;
        let imm53 = ((delta >> 3) & 7) as u8;
        let imm109 = ((delta >> 9) & 3) as u8;
        let imm20 = (delta & 7) as u8;
        let sign = if delta < 0 { 1 } else { 0 };
        self.asm_internal(Jal, sign, imm86, imm53, imm109 * 8 + imm20);
    }
    pub fn asm_store(&mut self, op: Anti80Opcode, rs1: Anti80Reg, rs2: Anti80Reg, offset: i16) {
        assert!(-32 <= offset && offset < 32); // XXX No IMM10 support yet
        let imm43 = ((offset >> 3) & 3) as u8;
        let imm20 = (offset & 7) as u8;
        let sign = if offset < 0 { 1 } else { 0 };
        self.asm_internal(op, sign, imm20, rs1 as u8, imm43 * 8 + (rs2 as u8))
    }

    pub fn asm_sw(&mut self, rs1: Anti80Reg, rs2: Anti80Reg, offset: i16) {
        self.asm_store(Sw, rs1, rs2, offset)
    }
    pub fn asm_sb(&mut self, rs1: Anti80Reg, rs2: Anti80Reg, offset: i16) {
        self.asm_store(Sb, rs1, rs2, offset)
    }
    pub fn asm_skipc(&mut self) {}
    pub fn asm_skipci(&mut self) {}
    pub fn asm_lw(&mut self) {}
    pub fn asm_lwi(&mut self) {}
    pub fn asm_lb(&mut self) {}
    pub fn asm_lbi(&mut self) {}
    pub fn asm_lbu(&mut self) {}
    pub fn asm_lbui(&mut self) {}
    pub fn asm_mov(&mut self) {}
    pub fn asm_movi(&mut self, dest: Anti80Reg, imm: i16) {
        self.asm_alui(Mov, dest, R0, imm)
    }

    pub fn asm_add(&mut self, dest: Anti80Reg, rs1: Anti80Reg, rs2: Anti80Reg) {
        self.asm_alu(Add, dest, rs1, rs2)
    }
    pub fn asm_addi(&mut self, dest: Anti80Reg, rs1: Anti80Reg, imm: i16) {
        self.asm_alui(Add, dest, rs1, imm)
    }
    pub fn asm_subr(&mut self) {}
    pub fn asm_subri(&mut self) {}
    pub fn asm_and(&mut self) {}
    pub fn asm_andi(&mut self) {}
    pub fn asm_or(&mut self) {}
    pub fn asm_ori(&mut self) {}
    pub fn asm_xor(&mut self) {}
    pub fn asm_xori(&mut self) {}
    pub fn asm_sra(&mut self) {}
    pub fn asm_srai(&mut self) {}
    pub fn asm_srl(&mut self) {}
    pub fn asm_srli(&mut self) {}
    pub fn asm_sl(&mut self) {}
    pub fn asm_sli(&mut self) {}

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
        let simm6 = if sign { src2 - 32 } else { src2 };

        let src2_is_reg = opcode == Sw || opcode == Sb || sign && src2 >= 0x18;
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
                let uimm11 = (src2 >> 3 << 9) + (dest << 6) + (src1 << 3) + (src2 & 7);
                let jal_simm12 = if sign { uimm11 - 2048 } else { uimm11 };
                self.pc += (jal_simm12 << 1) as i16;
            }
            Skipc => {
                use Anti80SkipCond::*;
                if match FromPrimitive::from_i16(dest).expect("can't happen") {
                    Beq => rs1 == rs2_imm,
                    Bne => rs1 != rs2_imm,
                    Blt => rs1 < rs2_imm,
                    Bge => rs1 >= rs2_imm,
                    Bltu => (rs1 as usize) < (rs2_imm as usize),
                    Bgeu => (rs1 as usize) >= (rs2_imm as usize),
                    _ => false,
                } {
                    self.pc += 2
                }
            }
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
        let mut anti80 = Anti80::new();

        anti80.asm_movi(R5, 14);
        anti80.asm_movi(R6, 7);
        anti80.asm_alu(Add, R7, R5, R6);

        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 0, 0]);

        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 7, 0]);

        anti80.step();
        assert_eq!(anti80.reg, [0, 0, 0, 0, 0, 14, 7, 21]);

        //      anti80.asm(Jal, 1, 7, 5, 6 | 0x18);
    }
}
