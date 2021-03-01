# The Anti80, the Anti-Z80

An intellectual exercise looking to answer the question: what would have been a better ISA for compilers (and humans), but using the same technology, area, and number of transistors as the Zilog Z80?

## The 16-instruction ISA

| name  | opcode 15:12 | sign 11 | dest 10:8      | src1 7:5 | src2 4:0           | Comments                                                                      |
| ----- | ------------ | ------- | -------------- | -------- | ------------------ | ----------------------------------------------------------------------------- |
| IMM10 |              | imm14   | nzresv:2,imm13 | imm12:10 | imm9:5             | Interlocking prefix, affects the immediate field of the following instruction |
| JAL   |              |         | imm8:6         | imm5:3   | imm10:9,imm2:0/rs2 | NB: sign&imm10&imm9 ? rs2 : signed immediate {sign,imm10:0}                   |
| SW    |              |         | imm2:0         | rs1      | imm4:3,rs2         |
| SB    |              |         | imm2:0         | rs1      | imm4:3,rs2         |
| SKIPC |              |         | cond           | rs1      | imm4:0/rs2         |
| LW    |              |         | rd             | rs1      | imm4:0/rs2         |
| LB    |              |         | rd             | rs1      | imm4:0/rs2         |
| LBU   |              |         | rd             | rs1      | imm4:0/rs2         |
| MOV   |              |         | rd             | nzresv   | imm4:0/rs2         | Load Immediate, register value is ignored                                     |
| ADD   |              |         | rd             | rs1      | imm4:0/rs2         |
| SUBR  |              |         | rd             | rs1      | imm4:0/rs2         |
| AND   |              |         | rd             | rs1      | imm4:0/rs2         |
| OR    |              |         | rd             | rs1      | imm4:0/rs2         |
| XOR   |              |         | rd             | rs1      | imm4:0/rs2         |
| SR    |              |         | rd             | rs1      | imm4:0/rs2         | imm3 ? sra : srl                                                              |
| SL    |              |         | rd             | rs1      | imm4:0/rs2         |

For most instruction, insn[15,4:3]|$past(inst)==IMM10 choses between the register and the immediate.  Also applies to JAL, but its larger immediate means we interp these bits differently.

nzresv = must be zero. Non-zero values are reserved

imm4:0/rs2 is immediate unless sign & imm4 are set, then it's a register

cond:
  reg: EQ,NE,LT,GE,LTU,GEU
  imm: EQ,NE,LT,GT,LTU,GTU

## Priorities

0. Must fit within the constaints of the Z80 implementation (number of transistors, silicon area, pins etc)
1. Very regular ISA, as few as possible special cases.
2. Make the encoding dense as possible (good use of instruction bits is critical when every instruction is read from memory)
3. Make the ISA cover as many frequently used instructions as possible
4. (Inspired by RISC-V) Make the encoding as cheap as possible, in particular, fields should not move around, especially critical path fields like register names or high-fanout immediate sign.  The current encoding gives access to register source field in the first-read byte.

## Credits

Inspiration taken from RISC-V, Yale Patt's LC-3, and IBM's PowerPC

Deeply indebt to Erik Corry for poking holes in many many earlier drafts and suggesting changes.  All remaining errors are mine.

### Properties
- The Z80 has 16 8-bit registers. Using the same bits and datapath Anti80 offer 8 16-bit registers.
- All instructions are exactly 16-bit and must be 16-bit aligned.  It's not possible to target an unaligned instruction (JAL addresses w)
- All instructions have optional immediate fields that can be expanded to full 16-bit with an immediately preceeding IMM10 instructions.
- The imm4:0/rs2 denotes a register unless the sign & imm4 & imm3 bits are set and there is no proceeding IMM10 instruction.  That means we exchange the -32..-25 immediate values for the ability to use a register as a source, leaving just -24..31.  Of course used IMM10 enlargen that to any full 16-bit constant.
- There are no condition flags
- There are only two control flow instructions, a conditional skip-next (SKIP) and a jump-and-link (JAL) instruction (like most instruction, it can also use a register).  JAL sets the link register r7 unconditionally.  This means that returning clobbers the return address, but that's  should be ok.  Skipping a IMM10 is fine, but rarely useful.
- As subtraction with an immediate can already be done by an ADD, Anti80 uses (inspired by PowerPC/POWER) a reverse ordered subtraction (arg2 - arg1).  As a special case with arg2 = 0 it implements negation.
- There's currently little thought given to interrupts; it will require more support
- The comparisons done by SKIP are still under investition.  The full set is ne,eq,lt,ge,le,gt,ltu,geu,leu,gtu.  RISC-V makes do with just ne,eq,lt,ge,ltu,geu because of the symmetry of the two operands means you can just swap them, but in Anti80 they aren't symmetrical.  The current design behaves differently for comparing against an immediate (as often you can replace a `x <= k` with `x < k-1` but not in all cases).
- LI ignores the operand that comes from the rs1 register so there's a slight inefficiency in the encoding.
- The Z80 alu is 4 bit wide and Anti80 is fundamentally 16-bit, thus expect 4 cycles through the ALU for (almost) all instructions.
- Anti80 follows RISC-V in many choices, but does allow a register + register load.
- Whether unaligned loads/stores work or trap is still TBD (learning towards trap)
- the JAL immediate is interpreted as relative to following instruction and shifted up (thus reaching ~ +/- 2048), but the register value is taken to be an absolute byte address and the LSB is ignored.