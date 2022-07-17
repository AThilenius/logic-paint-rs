# CPU

This is an 8-bit CPU modeled after Ben Eater's 8-bit CPU with some additions
(16-bit address space, 5-bit op codes, more ALU ops.

## Thoughts

This next section is just a place that I track thoughts/notes as I work on the
project. It isn't organized.

### ALU

ALU manages it's own registers A and B. Only their inputs are connected to the
main bus. Outputs are not buffered and go directly into ALU circuitry.

I would like the following 15 ops (4-bit control)

- ALU Out: EN
- Load A: LDA
- Load B: LDB
- Pass-through A: PTA (A)
- Pass-through B: PTB (B)
- Addition: ADD (A + B) => Bus
- Subtraction: SUB (A - B)
- One's Complement: NOT (!A)
- One's Complement: NOT (!B)
- Bin and: AND (A & B)
- Bin xor: XOR (A ^ B)
- Logical Shift Left: SL (A << 1)
- Logical Shit Right: SR (A >> 1)
- (Optional) Arithmetic Shift Right (sign extend): ASR (A >>> 1)

Output:

- Zero
- Carry Out

Don't need to implement in the ALU

- Bin or: OR (A | B)
  - You can just output two registers at once to the bus, and clock it in.
