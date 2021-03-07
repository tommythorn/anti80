# The Anti80, the Anti-Z80

Anti80 is a 16-bit ISA that should be entirely possible to realize
using the same technology, area, and number of transistors as the
Zilog Z80, but dramatically easier to use and expecially easier for
a compiler to target.

## Design Goals

0. Must fit within the constaints of the Z80 implementation (number of
   transistors, silicon area, pins etc)

1. Very regular ISA, as few as possible special cases, especially no
   register constraints.

2. Make the encoding dense as possible (good use of instruction bits
   is critical when every instruction is read from memory)

3. Make the ISA cover as many frequently used instructions as possible

4. (Inspired by RISC-V) Make the encoding as cheap as possible, in
   particular, fields should not move around, especially critical path
   fields like register names or high-fanout immediate sign.  The
   current encoding gives access to register source field in the
   first-read byte.


## Properties

- The Z80 has 16 8-bit registers. Using the same bits and datapath,
  Anti80 offer 8 16-bit registers.

- All instructions are exactly 16-bit and must be 16-bit aligned.
  It's not possible to target an unaligned instruction.

- All instructions have optional immediate fields that can be expanded
  to full 16-bit with an immediately preceeding `prefix` instructions.

- There are no condition flags.

- There are only two control flow instructions, a conditional
  skip-next (`skipc`) and a jump-and-link (`jal`) instruction (like
  most instruction, it can also use a register).

- The `jal` jumps to the address in the register given by `rs2` iff
  insn[15,4,3] are all set.  Otherwise it forms the 11-bit signed
  immediate to be the relative _instruction_ from the following
  instruction (that, is it's multiplied by 2 and added to pc + 2).
  Before transfering control, pc + 2 is written to `r7`.

  This means by itself, `jal` can reach instructions in a [-512, 2046]
  byte window (and everything if prefixed).

  Since `jal` is also used to return that means that the return address
  will be clobbered.

- As subtraction with an immediate can already be done by an `add`,
  Anti80 uses (inspired by PowerPC/POWER) a reverse ordered
  subtraction (arg2 - arg1).

- The Z80 alu is 4-bit wide and Anti80 is fundamentally 16-bit, thus
  expect 4 cycles through the ALU for (almost) all instructions.

- Whether unaligned loads/stores work or trap is still TBD (learning
  towards trap)

- Interrupt support is still TBD.  One option:
  
  Have a single interrupt enable bit (`IE`).
  Upon a taken interrupt, we clear `IE`, write the registers `r6`
  and `r7` to a defined memory location, and execute a `jal` to
  transfer to the interrupt service routine.
  To return, a modification of `jal` is used that loads `r7` from
  the special memory location rather than setting it to the previous
  PC.  (Other alternatives: spend more register state or simply
  reserve a register for interrupts.)

## The 15-instruction ISA

| 15:12 opcode | 11 sign | 10:8 dest       | 7:5 src1 | 4:0 src2    | Comments                                           |
| ------------ | ------- | --------------- | -------- | ----------- | -------------------------------------------------- |
| `prefix`     | nzresv  | nzresv,imm14:13 | imm12:10 | imm9:5      | Extends the following instruction                  |
|              |         |                 |          |             |
| `sw`         |         | imm2:0          | rs1      | imm4:3,rs2  | simm6 in -32..31                                   |
| `sb`         |         | imm2:0          | rs1      | imm4:3,rs2  | simm6 in -32..31                                   |
| `jal`        |         | imm2:0          | imm5:3   | imm10:6/rs2 | NB: insn[15,4:3] == 4 ? rs2 : next_pc + simm11 * 2 |
|              |         |                 |          |             |
| `li`         |         | rd              | imm7:5   | imm4:0      | simm8 = -256..255                                  |
| `lw`         |         | rd              | rs1      | imm4:0      |
| `lb`         |         | rd              | rs1      | imm4:0      |
| `lbu`        |         | rd              | rs1      | imm4:0      |
|              |         |                 |          |             |
| `skipc`      |         | cond            | rs1      | imm4:0/rs2  | simm6 in -24..31                                   |
| `add`        |         | rd              | rs1      | imm4:0/rs2  |
| `subr`       |         | rd              | rs1      | imm4:0/rs2  |
| `and`        |         | rd              | rs1      | imm4:0/rs2  |
| `or`         |         | rd              | rs1      | imm4:0/rs2  |
| `xor`        |         | rd              | rs1      | imm4:0/rs2  |
|              |         |                 |          |             |
| `shr`        |         | rd              | rs1      | imm4:0/rs2' | See below                                          |

nzresv = must be zero; non-zero values are reserved

immediate encoding variants: prefix, store, jal, li, load, alu, shift.

## Decoding rules

The minimize the decoding cost, almost all fields have a fixed
locations, notably registers designators rd, rs1, and rs2.

The exception is the immedate fields where only the sign is in a fixed location.
There seven cases:
- prefix uses insn[10:0] as the [14:5] of the immediate for the following instruction
- stores always have rs2 register argument and insn[15,10:8] as a signed immediate
- jal takes an absolute address from rs2 iff insn[15,4,3] == 4, other uses
  insn[15,4:0,7:5,10:8] as a signed 11-bit immediate field.
- li forms an 9-bit signed immediate from insn[15,7:0] 
- load always used insn[15,4:0] for the signed immediate
- alu uses rs2 iff insn[15,4,3] == 4 otherwise uses insn[15,4:0] as signed immediate.
- The shift instruction has a less regular encoding (trading simplicity for density):

| sign,imm4:0 | Decoding   |
| ----------- | ---------- |
| 00iiii      | `srl` imm4 |
| 01iiii      | `sra` imm4 |
| 100rrr      | `srl` rs2  |
| 101rrr      | `sll` rs2  |
| 110rrr      | `sra` rs2  |
| 111iii      | `sll` imm3 |

That is
- `sll` iff S & insn[3],
- `sra` or `srl` iff !S | !insn[3]
- `sra` iff SRx & insn[4]
- imm4 iff !S
- imm3 iff S & insn[4] & insn[3]

Unfortunately, the eight missing `sll` with 8,..,15 immediates have to
broken up into two instructions.

Condition codes are decoded as: `eq`,`ne`,-,-,`lt`,`ge`,`ltu`,`geu`.
E.g. `skipc r9, geu, 11` means that if `(unsigned)r9 >= 11u` then the next
instruction is ignored (if skipping a `prefix` then a total of two
instructions are skipped).

## Credits

Inspiration taken from RISC-V, Yale Patt's LC-3, and IBM's PowerPC

Deeply indebt to Erik Corry for poking holes in many many earlier
drafts and suggesting changes.  All remaining errors are mine.

## TODO
* Change `skipc` to `pred` (which implies flipping the meaning).  This is mostly to make RISC-V translations trivial.
* Fix the behavior of `pred` on `prefix`
* Complete the assembly and testing of every instruction
* Write small examples (Sieve, recursive fib, ...)
* (Eventually) rework the opcode & shift bitpatterns for cheaper decoding.

Nice to have, but not planned:
* Small compiler
* Timing simulator
* Verilog implementation/FPGA Softcore
