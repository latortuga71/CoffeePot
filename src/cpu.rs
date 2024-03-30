use std::panic;

use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    pub pc: u64,
    pub sp: u32,
    pub mmu: MMU,
    pub x_reg: [u64; 32],
    pub debug_flag: bool,
}

// XLEN = u64 arch size

// RV64I: base integer instructions
impl std::fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display_string = format!("PC: {:#08X}\nSP: {:#08X}\nREGISTERS:", self.pc, self.sp);
        for i in 0..32 {
            let s = format!("X_{:?}: {:#04X} ", i, self.x_reg[i]).to_string();
            display_string.push_str("\n");
            display_string.push_str(&s);
        }
        display_string.push_str("\n");
        write!(f, "{}", display_string)
    }
}

impl CPU {
    pub fn new() -> Self {
        let mut xreg: [u64; 32] = [0; 32];
        let sp_start = 0xFFFF;
        xreg[2] = sp_start;
        CPU {
            sp: sp_start as u32,
            pc: 0x00000000,
            mmu: MMU::new(),
            x_reg: xreg,
            debug_flag: true,
        }
    }
    // execute instructioon
    pub fn execute(self: &mut Self, instruction: u32) -> bool {
        // I TYPE
        let opcode = instruction & 0b1111111;
        let rd = (instruction & 0x00000f80) >> 7;
        let rs1 = (instruction & 0x000f8000) >> 15;
        let _rs2 = (instruction & 0x01f00000) >> 20;
        let funct3 = (instruction & 0x00007000) >> 12;
        let funct7 = (instruction & 0xfe000000) >> 25;
        let funct5 = (instruction & 0xfe000000) >> 27;
        let imm = ((instruction as i32 as i64) >> 20) as u64;
        let imm_5_11_mode = (imm >> 6) & 0b111111;
        let _imm_5_11 = imm & 0b111111;
        let shamt = imm & 0b111111;
        let _funct6 = funct7 >> 1;

        // S TYPE IMMEDIATE VALUE
        let imm115 = (instruction >> 25) & 0b1111111;
        let imm40 = (instruction >> 7) & 0b11111;
        let imm_s = (imm115 << 5) | imm40;
        let imm_s_type = ((imm_s as i32) << 20) >> 20;
        // B TYPE IMMEDIATE VALUE
        let imm12 = (instruction >> 31) & 1;
        let imm105 = (instruction >> 25) & 0b111111;
        let imm41 = (instruction >> 8) & 0b1111;
        let imm11 = (instruction >> 7) & 1;
        let imm_b = (imm12 << 12) | (imm11 << 11) | (imm105 << 5) | (imm41 << 1);
        let imm_b_type = ((imm_b as i32) << 19) >> 19;
        // J TYPE IMMEDIATE VALUE
        let imm20 = (instruction >> 31) & 1;
        let imm101 = (instruction >> 21) & 0b1111111111;
        let imm11 = (instruction >> 20) & 1;
        let imm1912 = (instruction >> 12) & 0b11111111;
        let imm_j = (imm20 << 20) | (imm1912 << 12) | (imm11 << 11) | (imm101 << 1);
        let imm_j_type = ((imm_j as i32) << 11) >> 11;
        // U TYPE IMMEDIATE VALUE
        let imm_u_type = (instruction as i32 as i64 as u64) >> 12;
        match opcode {
            0b0101111 => match funct3 {
                0x3 => match funct5 {
                    0x2 => self.load_double_word_atomic(rd, rs1),
                    0x3 => self.store_double_word_atomic(rd, rs1, _rs2),
                    0x1 => self.swap_double_word_atomic(rd, rs1, _rs2),
                    0x0 => self.add_double_word_atomic(rd, rs1, _rs2),
                    0xC => self.and_double_word_atomic(rd, rs1, _rs2),
                    0x8 => self.or_double_word_atomic(rd, rs1, _rs2),
                    0x4 => self.xor_double_word_atomic(rd, rs1, _rs2),
                    0x14 => self.max_double_word_atomic(rd, rs1, _rs2),
                    0x10 => self.min_double_word_atomic(rd, rs1, _rs2),
                    0x18 => self.minu_double_word_atomic(rd, rs1, _rs2),
                    0x1C => self.maxu_double_word_atomic(rd, rs1, _rs2),
                    _ => panic!("invalid funct5"),
                },
                0x2 => match funct5 {
                    0x2 => self.load_word_atomic(rd, rs1),
                    0x3 => self.store_word_atomic(rd, rs1, _rs2),
                    0x1 => self.swap_word_atomic(rd, rs1, _rs2),
                    0x0 => self.add_word_atomic(rd, rs1, _rs2),
                    0xC => self.and_word_atomic(rd, rs1, _rs2),
                    0x8 => self.or_word_atomic(rd, rs1, _rs2),
                    0x4 => self.xor_word_atomic(rd, rs1, _rs2),
                    0x14 => self.max_word_atomic(rd, rs1, _rs2),
                    0x10 => self.min_word_atomic(rd, rs1, _rs2),
                    0x18 => self.minu_word_atomic(rd, rs1, _rs2),
                    0x1C => self.maxu_word_atomic(rd, rs1, _rs2),
                    _ => panic!("Invalid funct5"),
                },
                _ => panic!("invalid funct3"),
            },
            0b0110011 => match funct3 {
                0x0 => match funct7 {
                    0x0 => self.add(rd, rs1, _rs2),
                    0x1 => self.mul(rd, rs1, _rs2),
                    0x20 => self.sub(rd, rs1, _rs2),
                    _ => panic!("Invalid funct7"),
                },
                0x5 => match funct7 {
                    0x0 => self.srl(rd, rs1, _rs2),
                    0x1 => self.divu(rd, rs1, _rs2),
                    0x20 => self.sra(rd, rs1, _rs2),
                    _ => panic!("Invalid funct7"),
                },
                0x4 => match funct7 {
                    0x0 => self.xor(rd, rs1, _rs2),
                    0x1 => self.div(rd, rs1, _rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x6 => match funct7 {
                    0x0 => self.or(rd, rs1, _rs2),
                    0x1 => self.rem(rd, rs1, _rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x7 => match funct7 {
                    0x0 => self.and(rd, rs1, _rs2),
                    0x1 => self.remu(rd, rs1, _rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x1 => match funct7 {
                    0x0 => self.sll(rd, rs1, _rs2),
                    0x1 => self.mulh(rd, rs1, _rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x2 => match funct7 {
                    0x0 => self.slt(rd, rs1, _rs2),
                    0x1 => self.mulhsu(rd, rs1, _rs2),
                    _ => panic!("invalid funct 7"),
                },
                0x3 => match funct7 {
                    0x0 => self.sltu(rd, rs1, _rs2),
                    0x1 => self.mulu(rd, rs1, _rs2),
                    _ => panic!("invalidu func7"),
                },
                _ => panic!("INVALID FUNCT3"),
            },
            // Load Instructions
            0b0000011 => match funct3 {
                // I TYPE
                0x0 => self.load_byte(rd, rs1, imm),
                0x1 => self.load_half(rd, rs1, imm),
                0x2 => self.load_word(rd, rs1, imm),
                0x4 => self.load_byte_u(rd, rs1, imm),
                0x5 => self.load_half_u(rd, rs1, imm),
                0x3 => self.load_double_word(rd, rs1, imm),
                0x6 => self.load_word_unsigned(rd, rs1, imm),
                _ => panic!("Invalid funct3"),
            },
            // Store Instructions
            0b0100011 => match funct3 {
                // Stores are S TYPE everything is the same except the immediate register
                0x0 => self.store_byte(_rs2, rs1, imm_s_type),
                0x1 => self.store_half(_rs2, rs1, imm_s_type),
                0x2 => self.store_word(_rs2, rs1, imm_s_type),
                0x3 => self.store_double_word(_rs2, rs1, imm_s_type),
                _ => panic!("Invalid funct3 {:#08X}", funct3),
            },
            0b0010011 => match funct3 {
                // I TYPE
                0x0 => self.addi(rd, rs1, imm),
                0x4 => self.xori(rd, rs1, imm),
                0x6 => self.ori(rd, rs1, imm),
                0x7 => self.andi(rd, rs1, imm),
                0x1 => self.slli(rd, rs1, shamt),
                0x5 => match imm_5_11_mode {
                    0x0 => self.srli(rd, rs1, shamt),
                    0x20 => self.srai(rd, rs1, shamt),
                    _ => panic!("INVALID IMMEDIATE 5-11"),
                },
                0x2 => self.slti(rd, rs1, imm),
                0x3 => self.sltiu(rd, rs1, imm),
                _ => panic!("Unimplemented funct3"),
            },
            0b1100011 => match funct3 {
                0x0 => self.beq(rs1, _rs2, imm_b_type),
                0x1 => self.bne(rs1, _rs2, imm_b_type),
                0x4 => self.blt(rs1, _rs2, imm_b_type),
                0x5 => self.bge(rs1, _rs2, imm_b_type),
                0x6 => self.bltu(rs1, _rs2, imm_b_type),
                0x7 => self.bgeu(rs1, _rs2, imm_b_type),
                _ => panic!("PANIC INVAID OPCODE"),
            },
            0b1101111 => self.jal(rd, imm_j_type),
            0b1100111 => self.jalr(rd, rs1, imm),
            0b0110111 => self.lui(rd, imm_u_type),
            0b0010111 => self.auipc(rd, imm_u_type),
            0b0011011 => match funct3 {
                0x0 => self.addiw(rd, rs1, imm),
                0x1 => self.slliw(rd, rs1, shamt),
                0x3 => match funct7 {
                    0x0 => self.srliw(rd, rs1, shamt),
                    0x20 => self.sraiw(rd, rs1, shamt),
                    _ => panic!("invalid funct7"),
                },
                _ => panic!("invalid funct"),
            },
            0b0111011 => match funct3 {
                0x7 => match funct7 {
                    0x1 => self.remuw(rd, rs1, _rs2),
                    _ => panic!(""),
                },
                0x6 => match funct7 {
                    0x1 => self.remw(rd, rs1, _rs2),
                    _ => panic!(""),
                },
                0x4 => match funct7 {
                    0x1 => self.divw(rd, rs1, _rs2),
                    _ => panic!(""),
                },
                0x0 => match funct7 {
                    0x0 => self.addw(rd, rs1, _rs2),
                    0x20 => self.subw(rd, rs1, _rs2),
                    0x1 => self.mulw(rd, rs1, _rs2),
                    _ => panic!("invalid funct7"),
                },
                0x1 => self.sllw(rd, rs1, _rs2),
                0x5 => match funct7 {
                    0x1 => self.divuw(rd, rs1, _rs2),
                    0x0 => self.srlw(rd, rs1, _rs2),
                    0x20 => self.sraw(rd, rs1, _rs2),
                    _ => panic!("invalid funct7"),
                },
                _ => panic!("unknown funct3"),
            },
            0b1110011 => match funct7 {
                0x0 => self.ecall(),
                0x1 => todo!("EBREAK"),
                _ => panic!("INVALID FUNC7 for ECALL OR EBREAK"),
            },
            _ => panic!("PC: {:#08X} Unimplemented OpCode {:#013b}", self.pc, opcode),
        }
        // match on opcode then match on func3?
    }

    fn remu(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("REMU");
        let left = self.x_reg[rs1 as usize];
        let right = self.x_reg[rs2 as usize];
        let result = if right == 0 {
            left
        } else {
            left.wrapping_rem(right)
        };
        self.x_reg[rd as usize] = result as u64;
        false
    }

    fn rem(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("REM");
        let left = self.x_reg[rs1 as usize] as i64;
        let right = self.x_reg[rs2 as usize] as i64;
        let result = if right == 0 {
            left
        } else {
            left.wrapping_rem(right)
        };
        self.x_reg[rd as usize] = result as u64;
        false
    }

    fn divu(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("DIVU");
        let left = self.x_reg[rs1 as usize];
        let right = self.x_reg[rs2 as usize];
        let result = if right == 0 {
            core::u64::MAX
        } else {
            left.wrapping_div(right)
        };
        self.x_reg[rd as usize] = result;
        false
    }

    fn div(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("DIV");
        let left = self.x_reg[rs1 as usize] as i64;
        let right = self.x_reg[rs2 as usize] as i64;
        let result = if right == 0 {
            -1
        } else {
            left.wrapping_div(right)
        };
        self.x_reg[rd as usize] = result as u64;
        false
    }

    fn mulu(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("MULU");
        let left = self.x_reg[rs1 as usize] as u64 as u128;
        let right = self.x_reg[rs2 as usize] as u64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn sll(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SLL");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] << (self.x_reg[rs2 as usize] & 0b11111);
        false
    }

    fn slt(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SLT");
        if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[rs2 as usize] as i64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn slli(&mut self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SLLI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] << shamt;
        false
    }

    fn srli(&mut self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SRLI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] >> shamt;
        false
    }

    fn srai(&mut self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SRAI");
        self.x_reg[rd as usize] = ((self.x_reg[rs1 as usize] as i64) >> shamt) as u64;
        false
    }

    fn slti(&mut self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("SRLTI");
        if (self.x_reg[rs1 as usize] as i64) < (imm as i64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltiu(&mut self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("SLTIU");
        if (self.x_reg[rs1 as usize] as u64) < imm {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltu(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SLTU");
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn mulhsu(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("MULHSU");
        let left = self.x_reg[rs1 as usize] as i64 as u128;
        let right = self.x_reg[rs2 as usize] as u64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn srl(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SRL");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] >> (self.x_reg[rs2 as usize] & 0b11111);
        false
    }

    fn sra(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SRA");
        self.x_reg[rd as usize] =
            self.x_reg[rs1 as usize] >> ((self.x_reg[rs2 as usize] & 0b11111) as i64) as u64;
        false
    }

    fn mulh(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("MULH");
        let left = self.x_reg[rs1 as usize] as i64 as u128;
        let right = self.x_reg[rs2 as usize] as i64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn mul(&mut self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("MUL");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_mul(self.x_reg[rs2 as usize]);
        false
    }

    fn sub(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SUB");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_sub(self.x_reg[rs2 as usize]);
        false
    }

    fn add(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        if self.debug_flag {
            println!(
                "ADD x{rd} ({:#08X}) x{rs1} ({:#08X}) x{rs2} ({:#08X})",
                self.x_reg[rd as usize], self.x_reg[rs1 as usize], self.x_reg[rs2 as usize]
            );
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_add(self.x_reg[rs2 as usize]);
        false
    }
    fn store_double_word(self: &mut Self, rs2: u32, rs1: u32, imm: i32) -> bool {
        println!("SD RS1 = {:#08X}", self.x_reg[rs1 as usize]);
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u64;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_word(self: &mut Self, rs2: u32, rs1: u32, imm: i32) -> bool {
        println!("SW");
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_half(self: &mut Self, rs2: u32, rs1: u32, imm: i32) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u16;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 2]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_byte(self: &mut Self, rs2: u32, rs1: u32, imm: i32) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u8;
        if self.debug_flag {
            println!("SB {:#08X} <- {:#08X}", _memory_address, value)
        }
        self.mmu.memory_segment[_memory_address as usize] = value;
        false
    }
    fn load_word_unsigned(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("LWU");
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    fn load_double_word(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("LD");
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    fn load_double_word_atomic(self: &mut Self, rd: u32, rs1: u32) -> bool {
        println!("LR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    fn store_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SC.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        self.x_reg[rd as usize] = 0;
        false
    }
    fn load_word_atomic(self: &mut Self, rd: u32, rs1: u32) -> bool {
        println!("LR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i64 as u64;
        self.x_reg[rd as usize] = result as i64 as u64;
        false
    }
    fn store_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SC.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        self.x_reg[rd as usize] = 0;
        false
    }

    fn maxu_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMAXU.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as u32 as u64;
        let min = if result as u64 > self.x_reg[rs2 as usize] {
            self.x_reg[rs2 as usize]
        } else {
            result
        };
        self.x_reg[rd as usize] = min;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn minu_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMINU.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as u32 as u64;
        let min = if result as u64 > self.x_reg[rs2 as usize] {
            self.x_reg[rs2 as usize]
        } else {
            result
        };
        self.x_reg[rd as usize] = min;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn min_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMIN.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let min = if result > self.x_reg[rs2 as usize] as i32 {
            self.x_reg[rs2 as usize] as i32
        } else {
            result
        };
        self.x_reg[rd as usize] = min as u64;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn max_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMAX.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let max = if result > self.x_reg[rs2 as usize] as i32 {
            result
        } else {
            self.x_reg[rs2 as usize] as i32
        };
        self.x_reg[rd as usize] = max as u64;
        let value_as_bytes = max.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn xor_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOXOR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 ^ self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn or_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOOR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 | self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn and_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOAND.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 & self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn add_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOADD.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 + self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn minu_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMINU.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let min = if result as u64 > self.x_reg[rs2 as usize] {
            self.x_reg[rs2 as usize]
        } else {
            result as u64
        };
        self.x_reg[rd as usize] = min as u64;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn maxu_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMAXU.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);

        let max = if result as u64 > self.x_reg[rs2 as usize] {
            result as u64
        } else {
            self.x_reg[rs2 as usize]
        };
        self.x_reg[rd as usize] = max as u64;
        let value_as_bytes = max.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn max_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMAX.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let max = if result as i64 > self.x_reg[rs2 as usize] as i64 {
            result as i64
        } else {
            self.x_reg[rs2 as usize] as i64
        };
        self.x_reg[rd as usize] = max as u64;
        let value_as_bytes = max.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn min_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOMIN.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let min = if result as i64 > self.x_reg[rs2 as usize] as i64 {
            self.x_reg[rs2 as usize] as i64
        } else {
            result as i64
        };
        self.x_reg[rd as usize] = min as u64;
        // write to the memory address
        let value_as_bytes = min.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn xor_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOXOR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 ^ self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn and_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOAND.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 & self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn or_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOOR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 | self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn add_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOADD.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 + self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn swap_double_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOSWAP.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value1 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.memory_segment[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.memory_segment[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.memory_segment[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        let oldrs2 = self.x_reg[rs2 as usize];
        self.x_reg[rs2 as usize] = result as u64;
        self.x_reg[rd as usize] = oldrs2;
        let swapped_rd = self.x_reg[rd as usize];
        // write to the memory address now
        let value_as_bytes = swapped_rd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn swap_word_atomic(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AMOSWAP.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        self.x_reg[rd as usize] = result as u64;
        let oldrs2 = self.x_reg[rs2 as usize];
        self.x_reg[rs2 as usize] = result as u64;
        self.x_reg[rd as usize] = oldrs2;
        let swapped_rd = self.x_reg[rd as usize];
        // write to the memory address now
        let value_as_bytes = swapped_rd.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn load_word(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 16 bit value
    fn load_half(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 8 bit value
    fn load_byte(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value = self.mmu.memory_segment[_memory_address as usize] as u8;
        if self.debug_flag {
            println!(
                "LB x{rd} ({:#08X}) {:#08X} -> ({:#08X})",
                self.x_reg[rd as usize], _memory_address, value
            );
        }

        self.x_reg[rd as usize] = value as u64;
        false
    }
    // load 16 bit value
    fn load_half_u(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]) as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 8 bit value
    fn load_byte_u(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value = self.mmu.memory_segment[_memory_address as usize] as u8;
        self.x_reg[rd as usize] = value as u64;
        false
    }
    // add immediate
    fn addi(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        if self.debug_flag {
            println!(
                "ADDI x{rd} ({:#08X}) x{rs1} ({:#08X}) imm ({:#08X})",
                self.x_reg[rd as usize], self.x_reg[rs1 as usize], imm
            );
        }
        self.x_reg[rd as usize] = imm.wrapping_add(self.x_reg[rs1 as usize]);
        false
    }
    fn andi(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("ANDI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & imm;
        false
    }
    fn ori(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("ORI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | imm;
        false
    }
    fn xori(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("XORI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ imm;
        false
    }
    fn and(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("AND");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & self.x_reg[rs2 as usize];
        false
    }
    fn or(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("OR");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | self.x_reg[rs2 as usize];
        false
    }
    fn xor(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("XOR");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ self.x_reg[rs2 as usize];
        false
    }

    fn bgeu(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("BGEU");
        if (self.x_reg[rs1 as usize] as u64) >= (self.x_reg[rs2 as usize] as u64) {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }

    fn bltu(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("bltu");
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }

    fn bge(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("bge");
        if (self.x_reg[rs1 as usize] as i64) >= (self.x_reg[rs2 as usize] as i64) {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }

    fn blt(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("blt");
        if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[rs2 as usize] as i64) {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }

    fn bne(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("bne");
        if self.x_reg[rs1 as usize] != self.x_reg[rs2 as usize] {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }
    fn beq(self: &mut Self, rs1: u32, rs2: u32, imm_b_type: i32) -> bool {
        println!("beq");
        if self.x_reg[rs1 as usize] == self.x_reg[rs2 as usize] {
            self.pc += imm_b_type as i64 as u64;
            return true;
        }
        false
    }
    fn jal(self: &mut Self, rd: u32, imm_j_type: i32) -> bool {
        println!("JAL");
        self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
        self.pc = self.pc.wrapping_add(imm_j_type as i64 as u64);
        true
    }
    fn jalr(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("JALR");
        self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
        self.pc = imm.wrapping_add(self.x_reg[rs1 as usize]); // PC = RS1 + IMM
        true
    }
    fn lui(self: &mut Self, rd: u32, imm_u_type: u64) -> bool {
        println!("LUI");
        self.x_reg[rd as usize] = imm_u_type as i64 as u64;
        false
    }
    fn auipc(self: &mut Self, rd: u32, imm_u_type: u64) -> bool {
        println!("AUIPC");
        self.x_reg[rd as usize] = (imm_u_type as i64 as u64).wrapping_add(self.pc);
        false
    }

    fn addiw(self: &mut Self, rd: u32, rs1: u32, imm: u64) -> bool {
        println!("ADDIW confirm works");
        let rs1__ = self.x_reg[rs1 as usize];
        self.x_reg[rd as usize] = rs1__.wrapping_add(imm) as i64 as u64;
        false
    }

    fn slliw(self: &mut Self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SLLIW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = shamt as u32;
        self.x_reg[rd as usize] = (left >> right) as u32 as i64 as u64;
        false
    }

    fn srliw(self: &mut Self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SRLIW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = shamt as u32;
        self.x_reg[rd as usize] = (left << right) as u32 as i64 as u64;
        false
    }

    fn sraw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SRAW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32 & 0b11111;
        self.x_reg[rd as usize] = (left >> right) as u32 as i64 as u64;
        false
    }
    fn sraiw(self: &mut Self, rd: u32, rs1: u32, shamt: u64) -> bool {
        println!("SRAIW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = shamt as i32;
        self.x_reg[rd as usize] = (left >> right) as u32 as i64 as u64;
        false
    }
    fn srlw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SRLW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32 & 0b11111;
        self.x_reg[rd as usize] = (left >> right) as u32 as u64;
        false
    }
    fn sllw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SLLW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32 & 0b11111;
        self.x_reg[rd as usize] = (left << right) as u32 as u64;
        false
    }

    fn divuw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("DIVUW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32;
        let result = if rs2 == 0 {
            core::u32::MAX
        } else {
            left.wrapping_div(right)
        };
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }

    fn mulw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("MULW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32;
        let result = left.wrapping_mul(right);
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }
    fn subw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("SUBW");
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_sub(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }
    fn addw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("ADDW");
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_add(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }

    fn divw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("DIVW");
        let left = self.x_reg[rs1 as usize] as i32;
        let right = self.x_reg[rs2 as usize] as i32;
        let result = if rs2 == 0 {
            -1
        } else {
            left.wrapping_div(right)
        };
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }
    fn remw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("REMW");
        let left = self.x_reg[rs1 as usize] as i32;
        let right = self.x_reg[rs2 as usize] as i32;
        let result = if rs2 == 0 {
            left
        } else {
            left.wrapping_rem(right)
        };
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }
    fn remuw(self: &mut Self, rd: u32, rs1: u32, rs2: u32) -> bool {
        println!("REMUW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32;
        let result = if rs2 == 0 {
            left
        } else {
            left.wrapping_rem(right)
        };
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }

    fn ecall(self: &mut Self) -> bool {
        // Emulate a syscall
        // x0 == 0
        // a0 == x10 -> contains return value
        // a0 to a5 are arguments ONLY 6 ARGS
        // return value is set in a0
        // a7 == x17 -> contains syscall number
        let syscall = self.x_reg[17];
        let _a0 = self.x_reg[10];
        let _a1 = self.x_reg[11];
        let _a2 = self.x_reg[12];
        let _a3 = self.x_reg[13];
        let _a4 = self.x_reg[14];
        let _a5 = self.x_reg[15];
        match syscall {
            0x40 => {
                let fd = _a0;
                let end = _a1 + _a2;
                let raw_bytes = &self.mmu.memory_segment[_a1 as usize..end as usize];
                unsafe {
                    let utf_bytes = core::str::from_utf8_unchecked(raw_bytes);
                    //let utf_bytes = core::str::from_utf8(raw_bytes).unwrap();
                    // currently only can handle stdout or stderr
                    if fd == 1 || fd == 2 {
                        // print bytes
                        print!("{}", utf_bytes);
                        // set return value
                        self.x_reg[10] = _a2;
                    } else {
                        panic!("Handle other file descriptors");
                    }
                }
            }
            0x5D => {
                println!("\n=== CoffeePot ExitSycall! ===");
                std::process::exit(_a0 as i32);
            }
            _ => {
                panic!("Unimplemented syscall");
            }
        }
        false
    }
}
