use modular_bitfield::prelude::*;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
// XXX assignment of numerical values to opcodes matters for the decoder and is still TBD
#[derive(BitfieldSpecifier, PartialEq, Eq)]
#[bits = 4]
pub enum Anti80Opcode {
    Prefix,
    Sw,
    Sb,
    Jal,
    Li,
    Lw,
    Lb,
    Lbu,
    Skipc,
    Add,
    Subr,
    And,
    Or,
    Xor,
    Shr,
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
    pub prefix_mask: i16,
    pub prefix_bits: i16,
}

impl Anti80 {
    pub fn new() -> Anti80 {
        Self {
            memory: vec![0; 65536],
            pc: 0,
            reg: vec![0; 8],
            asm_addr: 0,
            prefix_mask: !0,
            prefix_bits: 0,
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

    // XXX factors assembly out of Anti80
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
        self.asm_internal(op, 1, dest as u8, rs1 as u8, rs2 as u8)
    }

    pub fn asm_prefix(&mut self, imm: i16) {
        self.asm_internal(
            Prefix,
            0,
            ((imm >> 13) & 3) as u8,
            ((imm >> 10) & 7) as u8,
            ((imm >> 5) & 31) as u8,
        )
    }

    fn asm_alui(&mut self, op: Anti80Opcode, dest: Anti80Reg, rs1: Anti80Reg, imm: i16) {
        if !(-24 <= imm && imm < 32) {
            self.asm_prefix(imm);
        }
        let sign = if imm < 0 { 1 } else { 0 };
        self.asm_internal(op, sign, dest as u8, rs1 as u8, (imm & 31) as u8)
    }

    pub fn asm_store(&mut self, op: Anti80Opcode, rs1: Anti80Reg, rs2: Anti80Reg, offset: i16) {
        if !(-32 <= offset && offset < 32) {
            self.asm_prefix(offset);
        }
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

    pub fn asm_jal(&mut self, rs2: Anti80Reg) {
        self.asm_internal(Jal, 1, 0, 0, rs2 as u8);
    }

    pub fn asm_jali(&mut self, target: i16) {
        let mut delta = target.wrapping_sub(self.asm_addr);
        assert!((delta & 1) == 0);
        delta = delta / 2;
        if !(-512 <= delta && delta < 2047) {
            self.asm_prefix(delta);
        }
        let imm20 = (delta & 7) as u8;
        let imm53 = ((delta >> 3) & 7) as u8;
        let imm106 = ((delta >> 6) & 31) as u8;
        let sign = if delta < 0 { 1 } else { 0 };
        self.asm_internal(Jal, sign, imm20, imm53, imm106);
    }

    pub fn asm_li(&mut self, dest: Anti80Reg, imm: i16) {
        if !(-256 <= imm && imm < 256) {
            self.asm_prefix(imm);
        }
        let sign = if imm < 0 { 1 } else { 0 };
        self.asm_internal(Li, sign, dest as u8, (imm as u8 >> 5) & 7, imm as u8 & 31)
    }

    pub fn asm_skipc(&mut self) {}
    pub fn asm_skipci(&mut self) {}
    pub fn asm_lw(&mut self) {}
    pub fn asm_lwi(&mut self) {}
    pub fn asm_lb(&mut self) {}
    pub fn asm_lbi(&mut self) {}
    pub fn asm_lbu(&mut self) {}
    pub fn asm_lbui(&mut self) {}

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

        let opcode = insn.opcode();
        let dest = insn.dest() as i16;
        let src2 = insn.src2() as i16;
        let src1 = insn.src1() as i16;
        let rd = dest as usize;
        let sign: i16 = if insn.sign() != 0 { -1 } else { 0 };

        let rs1 = self.reg[src1 as usize] as i16;
        let rs2 = self.reg[src2 as usize & 7] as i16;

        // Skipc, .., xor uses the same format
        let simm6_or_rs2 = if self.prefix_mask == !0 && src2 < 8 && sign < 0 {
            rs2
        } else {
            (sign << 5 | src2) & self.prefix_mask | self.prefix_bits
        };

        match &opcode {
            Prefix => {
                assert_eq!(self.prefix_mask, !0); // Prefix can only prefix a non-prefix
                self.prefix_mask = !0b0111_1111_1110_0000; // 14:5
                self.prefix_bits = dest << 13 | src1 << 10 | src2 << 5;
            }

            Sw | Sb => {
                let store_imm =
                    ((sign << 5) | (src2 >> 3) | dest) & self.prefix_mask | self.prefix_bits;

                match opcode {
                    Sb => self.store8(store_imm + rs1, rs2),
                    _ => {
                        self.store8(store_imm + rs1, rs2);
                        self.store8(store_imm + rs1 + 1, rs2 >> 8);
                    }
                }
            }

            Jal => {
                self.reg[7] = self.pc;
                if src2 < 0x18 && sign < 0 {
                    self.pc = rs2
                } else {
                    let jal_imm: i16 = (sign << 11 | src2 << 6 | src1 << 3 | dest)
                        & self.prefix_mask
                        | self.prefix_bits;
                    self.pc = self.pc + (jal_imm << 1)
                };
            }

            Li => {
                self.reg[rd] = (sign << 8 | src1 << 5 | src2) & self.prefix_mask | self.prefix_bits
            }

            Lw | Lb | Lbu => {
                let load_imm = (sign << 5 | src2) & self.prefix_mask | self.prefix_bits;
                match &opcode {
                    Lw => {
                        let lo = self.load8(rs1 + load_imm);
                        let hi = self.load8(rs1 + load_imm + 1);
                        self.reg[rd] = hi * 256 + lo;
                    }
                    Lb => {
                        let b = self.load8(rs1 + load_imm);
                        self.reg[rd] = if b < 128 { b } else { b - 256 }
                    }
                    Lbu => self.reg[rd] = self.load8(rs1 + load_imm),
                    _ => panic!("can't happen"),
                }
            }

            Skipc => {
                use Anti80SkipCond::*;
                if match FromPrimitive::from_i16(dest).expect("can't happen") {
                    Beq => rs1 == simm6_or_rs2,
                    Bne => rs1 != simm6_or_rs2,
                    Blt => rs1 < simm6_or_rs2,
                    Bge => rs1 >= simm6_or_rs2,
                    Bltu => (rs1 as usize) < (simm6_or_rs2 as usize),
                    Bgeu => (rs1 as usize) >= (simm6_or_rs2 as usize),
                    _ => false,
                } {
                    self.pc += 2
                }
            }

            Add => self.reg[rd] = simm6_or_rs2.wrapping_add(rs1),
            Subr => self.reg[rd] = simm6_or_rs2.wrapping_sub(rs1),
            And => self.reg[rd] = simm6_or_rs2 & rs1,
            Or => self.reg[rd] = simm6_or_rs2 | rs1,
            Xor => self.reg[rd] = simm6_or_rs2 ^ rs1,
            Shr => {
                self.reg[rd] = match insn.sign() as i16 * 4 + src2 >> 3 {
                    0 | 1 => ((rs1 as u16) >> (src2 & 15)) as i16, // SRL imm4
                    2 | 3 => rs1 >> (src2 & 15),                   // SRA imm4
                    4 => ((rs1 as u16) >> rs2) as i16,             // SRL rs2
                    5 => rs1 << rs2,                               // SLL rs2
                    6 => rs1 >> rs2,                               // SRA rs2
                    7 => rs1 << (src2 & 7),                        // SLL imm3
                    _ => panic!("can't happen"),
                }
            }
        };

        if opcode != Prefix {
            self.prefix_mask = !0;
            self.prefix_bits = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn li_all_values() {
        let mut cpu = Anti80::new();
        assert_eq!(cpu.reg, [0, 0, 0, 0, 0, 0, 0, 0]);

        for v in -32768..=32767 {
            cpu.asm_addr = 0;
            cpu.pc = 0;
            cpu.asm_li(R6, v);
            if (Anti80Insn::from_bytes(cpu.load16(cpu.pc))).opcode() == Prefix {
                cpu.step();
            }
            cpu.step();
            assert_eq!(cpu.reg, [0, 0, 0, 0, 0, 0, v, 0]);
        }
    }

    #[test]
    fn addi_all_values() {
        let mut cpu = Anti80::new();
        assert_eq!(cpu.reg, [0, 0, 0, 0, 0, 0, 0, 0]);
        cpu.reg[2] = 42;

        for v in -32768..=32767 {
            cpu.asm_addr = 0;
            cpu.pc = 0;
            cpu.asm_addi(R4, R2, v);
            if (Anti80Insn::from_bytes(cpu.load16(cpu.pc))).opcode() == Prefix {
                cpu.step();
            }
            cpu.step();
            assert_eq!(cpu.reg, [0, 0, 42, 0, v.wrapping_add(42) as i16, 0, 0, 0]);
        }
    }

    #[test]
    fn init_and_step() {
        let mut cpu = Anti80::new();

        cpu.asm_alu(Add, R7, R5, R6);

        cpu.step();
        assert_eq!(cpu.reg, [0, 0, 0, 0, 0, 14, 7, 0]);

        cpu.step();
        assert_eq!(cpu.reg, [0, 0, 0, 0, 0, 14, 7, 21]);

        //      anti80.asm(Jal, 1, 7, 5, 6 | 0x18);
    }
}
