use std::mem::transmute;

struct LC3B {
    /// Memory for the register file
    pub reg_file: [u16; 8],

    /// Program Counter
    pub pc: u16,

    /// Instruction register.
    pub ir: u16,

    /// Memory Address Register
    pub mar: u16,

    /// NZP flags
    pub n: bool,
    pub z: bool,
    pub p: bool,

    /// Buses
    pub d_bus: u16,
    pub a_bus: u16,
    pub b_bus: u16,

    /// Addressable memory
    pub memory: [u8; 65_536],
}

/// Notes
/// - All *_word_* values are lshifted 1 bit.
/// - There are no signed values in the sim, because two-complement adds together the same either
///   way.
/// - However! Some values are sign-extended, and some are not. The ones sign-extended are
///   semantically signed.
pub enum OpCode {
    ///
    BR {
        n: bool,
        z: bool,
        p: bool,
        // Sign extended
        pc_word_offset: u16,
    },
    JMP {
        base_r: usize,
    },
    Add(AluOperand),
    Sub(AluOperand),
    And(AluOperand),
    Or(AluOperand),
    Xor(AluOperand),
    LShift(u16),
    RShiftLog(u16),
    RShiftArith(u16),
    LDB {
        dr: usize,
        base_r: usize,
        // Sign extended
        offset: u16,
    },
    LDW {
        dr: usize,
        base_r: usize,
        // Sign extended
        offset: u16,
    },
    LDI {
        dr: usize,
        // Sign extended
        value: u16,
    },
    LEA {
        dr: usize,
        // Sign extended
        pc_offset: u16,
    },
    STB {
        sr: usize,
        base_r: usize,
        // Sign extended
        offset: u16,
    },
    STW {
        sr: usize,
        base_r: usize,
        // Sign extended
        offset: u16,
    },
}

impl TryFrom<u16> for OpCode {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        let opcode = (value & 0xF000) >> 12;

        Ok(match opcode {
            0 => OpCode::BR {
                n: value & 0b0000_1000_0000_0000 != 0,
                z: value & 0b0000_0100_0000_0000 != 0,
                p: value & 0b0000_0010_0000_0000 != 0,
                pc_word_offset: sign_extend::<9>(value),
            },
            12 => OpCode::JMP {
                base_r: sr1_base_r(value),
            },
            1 => {
                // Bit 5 is Register(0), Immediate(1)
                let operand = if value & 0b0000_0000_0010_0000 == 0 {
                    AluOperand::Registers {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        sr2: sr2(value),
                    }
                } else {
                    AluOperand::Immediate {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        immediate: sign_extend::<4>(value),
                    }
                };

                let bit_4 = value & 0b0000_0000_0001_0000 != 0;

                if !bit_4 {
                    OpCode::Add(operand)
                } else {
                    OpCode::Sub(operand)
                }
            }
            5 => {
                // Bit 5 is Register(0), Immediate(1)
                let operand = if value & 0b0000_0000_0010_0000 == 0 {
                    AluOperand::Registers {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        sr2: sr2(value),
                    }
                } else {
                    AluOperand::Immediate {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        immediate: value & 0xF,
                    }
                };

                let bit_4 = value & 0b0000_0000_0001_0000 != 0;

                if !bit_4 {
                    OpCode::And(operand)
                } else {
                    OpCode::Or(operand)
                }
            }
            9 => {
                // Bit 5 is Register(0), Immediate(1)
                let operand = if value & 0b0000_0000_0010_0000 == 0 {
                    AluOperand::Registers {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        sr2: sr2(value),
                    }
                } else {
                    AluOperand::Immediate {
                        dr: dr_sr(value),
                        sr1: sr1_base_r(value),
                        immediate: value & 0xF,
                    }
                };

                let bit_4 = value & 0b0000_0000_0001_0000 != 0;

                if !bit_4 {
                    OpCode::Xor(operand)
                } else {
                    return Err(());
                }
            }
            13 => {
                let bit_4 = value & 0b0000_0000_0001_0000 != 0;
                let bit_5 = value & 0b0000_0000_0010_0000 != 0;
                let amount = value & 0b0000_0000_0000_1111;

                match (bit_5, bit_4) {
                    (false, false) => OpCode::LShift(amount),
                    (false, true) => OpCode::RShiftLog(amount),
                    (true, true) => OpCode::RShiftArith(amount),
                    _ => return Err(()),
                }
            }
            2 => OpCode::LDB {
                dr: dr_sr(value),
                base_r: sr1_base_r(value),
                offset: unsafe { transmute(sign_extend::<6>(value)) },
            },
            6 => OpCode::LDW {
                dr: dr_sr(value),
                base_r: sr1_base_r(value),
                offset: unsafe { transmute(sign_extend::<7>(value << 1)) },
            },
            10 => OpCode::LDI {
                dr: dr_sr(value),
                value: unsafe { transmute(sign_extend::<9>(value)) },
            },
            14 => OpCode::LEA {
                dr: dr_sr(value),
                pc_offset: unsafe { transmute(sign_extend::<10>(value << 1)) },
            },
            3 => OpCode::STB {
                sr: dr_sr(value),
                base_r: sr1_base_r(value),
                offset: unsafe { transmute(sign_extend::<6>(value)) },
            },
            7 => OpCode::STW {
                sr: dr_sr(value),
                base_r: sr1_base_r(value),
                offset: unsafe { transmute(sign_extend::<7>(value << 1)) },
            },
            _ => return Err(()),
        })
    }
}

pub enum AluOperand {
    Registers {
        dr: usize,
        sr1: usize,
        sr2: usize,
    },
    Immediate {
        dr: usize,
        sr1: usize,
        immediate: u16,
    },
}

impl LC3B {
    // Executes the next instruction
    pub fn step(&mut self) -> Result<(), ()> {
        // State 16
        self.mar = self.pc;

        // State 18
        self.pc = self.pc.wrapping_add(2);

        // State 19
        let mar = self.mar as usize;
        self.ir = ((self.memory[mar] as u16) << 8) | self.memory[mar + 1] as u16;

        // State 20
        match OpCode::try_from(self.ir)? {
            // State 0
            OpCode::BR {
                n,
                z,
                p,
                pc_word_offset,
            } => {
                if (n && self.n) || (z && self.z) || (p && self.p) {
                    // Branch followed: pc = pc + pc_word_offset
                    self.d_bus = self.pc.wrapping_add(pc_word_offset);
                    self.pc = self.d_bus;
                } else {
                    // Branch not followed: pc = pc + 2
                    self.d_bus = self.pc.wrapping_add(2);
                    self.pc = self.d_bus;
                }
            }
            // State 12
            OpCode::JMP { base_r } => {
                self.d_bus = self.reg_file[base_r];
                self.pc = self.d_bus;
            }
            // State 1
            OpCode::Add(alu_operand) => {
                match alu_operand {
                    AluOperand::Registers { dr, sr1, sr2 } => {
                        self.d_bus = self.reg_file[sr1].wrapping_add(self.reg_file[sr2]);
                        self.reg_file[dr] = self.d_bus
                    }
                    AluOperand::Immediate { dr, sr1, immediate } => {
                        self.d_bus = self.reg_file[sr1].wrapping_add(immediate);
                        self.reg_file[dr] = self.d_bus
                    }
                }

                self.set_cc();
            }
            // State 1
            OpCode::Sub(alu_operand) => {
                match alu_operand {
                    AluOperand::Registers { dr, sr1, sr2 } => {
                        self.d_bus = self.reg_file[sr1].wrapping_sub(self.reg_file[sr2]);
                        self.reg_file[dr] = self.d_bus;
                    }
                    AluOperand::Immediate { dr, sr1, immediate } => {
                        self.d_bus = self.reg_file[sr1].wrapping_sub(immediate);
                        self.reg_file[dr] = self.d_bus;
                    }
                }

                self.set_cc();
            }
            // State 5
            OpCode::And(alu_operand) => {
                match alu_operand {
                    AluOperand::Registers { dr, sr1, sr2 } => {
                        self.d_bus = self.reg_file[sr1] & self.reg_file[sr2];
                        self.reg_file[dr] = self.d_bus;
                    }
                    AluOperand::Immediate { dr, sr1, immediate } => {
                        self.d_bus = self.reg_file[sr1] & immediate;
                        self.reg_file[dr] = self.d_bus;
                    }
                }

                self.set_cc();
            }
            // State 5
            OpCode::Or(alu_operand) => {
                match alu_operand {
                    AluOperand::Registers { dr, sr1, sr2 } => {
                        self.d_bus = self.reg_file[sr1] | self.reg_file[sr2];
                        self.reg_file[dr] = self.d_bus;
                    }
                    AluOperand::Immediate { dr, sr1, immediate } => {
                        self.d_bus = self.reg_file[sr1] | immediate;
                        self.reg_file[dr] = self.d_bus;
                    }
                }

                self.set_cc();
            }
            // State 9
            OpCode::Xor(alu_operand) => {
                match alu_operand {
                    AluOperand::Registers { dr, sr1, sr2 } => {
                        self.d_bus = self.reg_file[sr1] ^ self.reg_file[sr2];
                        self.reg_file[dr] = self.d_bus;
                    }
                    AluOperand::Immediate { dr, sr1, immediate } => {
                        self.d_bus = self.reg_file[sr1] ^ immediate;
                        self.reg_file[dr] = self.d_bus;
                    }
                }

                self.set_cc();
            }
            OpCode::LShift(_) => todo!(),
            OpCode::RShiftLog(_) => todo!(),
            OpCode::RShiftArith(_) => todo!(),
            // State 2
            OpCode::LDB { dr, base_r, offset } => {
                // State 2
                self.mar = self.reg_file[base_r].wrapping_add(offset);
                let mar = self.mar as usize;
                // State 21
                self.d_bus = self.memory[mar] as u16;
                self.reg_file[dr] = self.d_bus;
                self.set_cc();
            }
            OpCode::LDW { dr, base_r, offset } => {
                // State 6
                self.mar = self.reg_file[base_r].wrapping_add(offset);
                let mar = self.mar as usize;
                // State 22
                self.d_bus = ((self.memory[mar] as u16) << 8) | (self.memory[mar + 1] as u16);
                self.reg_file[dr] = self.d_bus;
                self.set_cc();
            }
            // State 10
            OpCode::LDI { dr, value } => {
                self.d_bus = value;
                self.reg_file[dr] = self.d_bus;
                self.set_cc();
            }
            // State 14
            OpCode::LEA { dr, pc_offset } => {
                self.d_bus = self.pc.wrapping_add(pc_offset);
                self.reg_file[dr] = self.d_bus;
                self.set_cc();
            }
            // State 3
            OpCode::STB { sr, base_r, offset } => {
                // State 3
                self.mar = self.reg_file[base_r].wrapping_add(offset);
                let mar = self.mar as usize;
                // State 23
                self.memory[mar] = (self.reg_file[sr] & 0xFF) as u8;
            }
            // State 7
            OpCode::STW { sr, base_r, offset } => {
                // State 7
                self.mar = self.reg_file[base_r].wrapping_add(offset);
                let mar = self.mar as usize;
                // State 24
                self.memory[mar] = (self.reg_file[sr] >> 8) as u8;
                self.memory[mar + 1] = (self.reg_file[sr] & 0xFF) as u8;
            }
        }

        Ok(())
    }

    #[inline(always)]
    fn set_cc(&mut self) {
        self.n = false;
        self.z = false;
        self.p = false;

        let signed: i16 = unsafe { transmute(self.d_bus) };

        if signed > 0 {
            self.p = true;
        } else if signed < 0 {
            self.n = true;
        } else {
            self.z = true;
        }
    }
}

fn main() {
    // Program needs to start at 0x2

    println!("Hello, world!");
}

#[inline(always)]
fn dr_sr(value: u16) -> usize {
    ((value & 0b0000_1110_0000_0000) >> 9) as usize
}

#[inline(always)]
fn sr1_base_r(value: u16) -> usize {
    ((value & 0b0000_0001_1100_0000) >> 6) as usize
}

#[inline(always)]
fn sr2(value: u16) -> usize {
    (value & 0b0000_0000_0000_0111) as usize
}

#[inline(always)]
fn sign_extend<const B: usize>(value: u16) -> u16 {
    // Ensure B is in a valid range (1..=16) at compile time
    debug_assert!(B >= 1 && B <= 16);

    let lower_bit_mask = (1 << B) - 1;
    let value = value & lower_bit_mask;

    // Extract the B-th bit (sign bit of the B-bit number)
    let sign_bit = (value >> (B - 1)) & 1;

    // If the sign bit is set, create a mask for the upper bits
    if sign_bit != 0 {
        let mask = !0u16 << B;
        value | mask
    } else {
        value
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sign_extend_9bit() {
        // Test positive 9-bit number
        assert_eq!(sign_extend::<9>(0b0_1010_1010), 0b0_1010_1010);

        // Test negative 9-bit number
        let input = 0b1_1010_1010; // 9-bit: -86
        assert_eq!(sign_extend::<9>(input), unsafe { transmute(-86_i16) });

        // Verify it keeps higher bits clear
        assert_eq!(sign_extend::<9>(0b1111_1010_1010), unsafe {
            transmute(-86_i16)
        });
    }

    #[test]
    fn test_sign_extend_6bit() {
        // Test positive 6-bit number
        assert_eq!(sign_extend::<6>(0b01_1010), 0b01_1010);

        // Test negative 6-bit number
        let input = 0b11_1010; // 6-bit: -6
        assert_eq!(sign_extend::<6>(input), unsafe { transmute(-6_i16) });

        // Test with junk bits
        assert_eq!(sign_extend::<6>(0b1010_1010), unsafe { transmute(-22_i16) });
        assert_eq!(sign_extend::<6>(0b1000_1010), 0b0000_0000_0000_1010);
    }
}
