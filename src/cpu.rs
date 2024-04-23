use std::io::BufRead;
use std::io::Write;

use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    pub pc: u64,
    pub sp: u64,
    pub mmu: MMU,
    pub x_reg: [u64; 32],
    pub f_reg: [u64; 32],
    pub csr_reg: [u64; 4096],
    pub current_compressed: bool,
    pub was_last_compressed: bool,
    pub debug_flag: bool,
}

// XLEN = u64 arch size
pub const DRAM_BASE: u64 = 0x8000_0000;
pub const DRAM_SIZE: u64 = 1024 * 1024 * 1024;
// RV64I: base integer instructions
impl std::fmt::Display for CPU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut display_string = format!("PC: {:#08X}\nSP: {:#08X}\nREGISTERS:", self.pc, self.sp);
        for i in 0..32 {
            let s = format!("X_{:?}: {:#08X} ", i, self.x_reg[i]).to_string();
            display_string.push_str(" ");
            display_string.push_str(&s);
            if i % 10 == 0 {
                display_string.push_str("\n");
            }
        }
        display_string.push_str("\n");
        /*
                let stack_slice = &self.mmu.virtual_memory[self.sp as usize..self.sp as usize + 32];
                let stack_format = format!("{:#08X?}\n", stack_slice);
                display_string.push_str(&stack_format);
        */
        write!(f, "{}", display_string)
    }
}

impl CPU {
    pub fn new() -> Self {
        let mut xreg: [u64; 32] = [0; 32];
        let stack_base = 0x0;
        let sp_start = stack_base + 1024 * 1024;
        xreg[2] = sp_start as u64;
        let mut sp = xreg[2];
        let og = sp;
        let mut mmu = MMU::new();

        mmu.virtual_memory[0x99..0x99 + 7].fill(0x41); // set to ascii 'AAAAAAAAA'
                                                       // setup initial program stack state
                                                       // Auxp
        let argc = u64::to_le_bytes(0x1);
        mmu.virtual_memory[sp as usize..sp as usize + 8].copy_from_slice(&argc);
        sp += 8;
        let argv0_addr = u64::to_le_bytes(0x99);
        mmu.virtual_memory[sp as usize..sp as usize + 8].copy_from_slice(&argv0_addr);
        sp += 8;
        mmu.virtual_memory[sp as usize..sp as usize + 7].fill(0x0);
        sp += 8;
        mmu.virtual_memory[sp as usize..sp as usize + 7].fill(0x0);
        sp += 8;
        mmu.virtual_memory[sp as usize..sp as usize + 7].fill(0x0);
        CPU {
            sp: og,
            pc: 0x00000000,
            mmu: mmu,
            x_reg: xreg,
            f_reg: [0; 32],
            csr_reg: [0; 4096],
            current_compressed: false,
            was_last_compressed: false,
            debug_flag: true,
        }
    }

    // https://github.com/d0iasm/rvemu/blob/main/src/cpu.rs <- guide because its confusing
    pub fn execute_compressed(self: &mut Self, instruction: u64) -> bool {
        let opcode = instruction & 0b11;
        let funct3 = (instruction >> 13) & 0x7;
        // opcodes are quadrants
        match opcode {
            // quadrant 0
            0b00 => match funct3 {
                0x0 => {
                    let rd = ((instruction >> 2) & 0x7) + 8;
                    let nzuimm = ((instruction >> 1) & 0x3c0) // znuimm[9:6]
                            | ((instruction >> 7) & 0x30) // znuimm[5:4]
                            | ((instruction >> 2) & 0x8) // znuimm[3]
                            | ((instruction >> 4) & 0x4); // znuimm[2]
                    if nzuimm == 0 {
                        panic!("illegal instruction");
                    }
                    self.c_add4spn(rd, nzuimm)
                }
                0x1 => {
                    let rd = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    let offset = ((instruction << 1) & 0xc0) // imm[7:6]
                            | ((instruction >> 7) & 0x38); // imm[5:3]{
                    self.c_fld(rd, rs1, offset)
                }
                0x2 => {
                    let rd = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    // offset[5:3|2|6] = isnt[12:10|6|5]
                    let offset = ((instruction << 1) & 0x40) // imm[6]
                            | ((instruction >> 7) & 0x38) // imm[5:3]
                            | ((instruction >> 4) & 0x4); // imm[2]
                    self.c_lw(rd, rs1, offset)
                }
                0x3 => {
                    let rd = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    let offset = ((instruction << 1) & 0xc0) | ((instruction >> 7) & 0x38);
                    self.c_ld(rd, rs1, offset)
                }
                0x4 => panic!("reserved"),
                0x5 => {
                    let rs2 = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    let offset = ((instruction << 1) & 0xc0) // imm[7:6]
                            | ((instruction >> 7) & 0x38); // imm[5:3]
                    self.c_fsd(rs1, rs2, offset)
                }
                0x6 => {
                    let rs2 = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    let offset = ((instruction << 1) & 0x40) // imm[6]
                            | ((instruction >> 7) & 0x38) // imm[5:3]
                            | ((instruction >> 4) & 0x4); // imm[2]
                    self.c_sw(rs2, rs1, offset)
                }
                0x7 => {
                    let rs2 = ((instruction >> 2) & 0x7) + 8;
                    let rs1 = ((instruction >> 7) & 0x7) + 8;
                    let offset = ((instruction << 1) & 0xc0) // imm[7:6]
                                | ((instruction >> 7) & 0x38); // imm[5:3]
                    self.c_sd(rs2, rs1, offset)
                }
                _ => todo!("quadrant 0 invalid funct3"),
            },
            // quadrant 1
            0b01 => match funct3 {
                0x0 => {
                    let rd = (instruction >> 7) & 0x1f;
                    let mut nzimm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    nzimm = match (nzimm & 0x20) == 0 {
                        true => nzimm,
                        false => (0xc0 | nzimm) as i8 as i64 as u64,
                    };
                    self.c_addi(rd, nzimm)
                }
                0x1 => {
                    let rd = (instruction >> 7) & 0x1f;
                    let mut nzimm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    nzimm = match (nzimm & 0x20) == 0 {
                        true => nzimm,
                        false => (0xc0 | nzimm) as i8 as i64 as u64,
                    };
                    self.c_addiw(rd, nzimm)
                }
                0x2 => {
                    let rd = (instruction >> 7) & 0x1f;
                    let mut nzimm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    nzimm = match (nzimm & 0x20) == 0 {
                        true => nzimm,
                        false => (0xc0 | nzimm) as i8 as i64 as u64,
                    };
                    self.c_li(rd, nzimm)
                }
                0x3 => {
                    let rd = (instruction >> 7) & 0x1f;
                    match rd {
                        0 => false,
                        2 => {
                            let mut nzimm = ((instruction >> 3) & 0x200) // nzimm[9]
                                    | ((instruction >> 2) & 0x10) // nzimm[4]
                                    | ((instruction << 1) & 0x40) // nzimm[6]
                                    | ((instruction << 4) & 0x180) // nzimm[8:7]
                                    | ((instruction << 3) & 0x20); // nzimm[5]
                            nzimm = match (nzimm & 0x200) == 0 {
                                true => nzimm,
                                false => (0xfc00 | nzimm) as i16 as i32 as i64 as u64,
                            };
                            self.c_addi16sp(rd, nzimm)
                        }
                        _ => {
                            let mut nzimm =
                                ((instruction << 5) & 0x20000) | ((instruction << 10) & 0x1f000);
                            // Sign-extended.
                            nzimm = match (nzimm & 0x20000) == 0 {
                                true => nzimm as u64,
                                false => (0xfffc0000 | nzimm) as i32 as i64 as u64,
                            };
                            self.c_lui(rd, nzimm)
                        }
                    }
                }
                0x4 => {
                    let funct2 = (instruction >> 10) & 0x3;
                    match funct2 {
                        0x0 => {
                            let rd = ((instruction >> 7) & 0b111) + 8;
                            let shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                            self.c_srli(rd, shamt)
                        }
                        0x1 => {
                            let rd = ((instruction >> 7) & 0b111) + 8;
                            let shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                            self.c_srai(rd, shamt)
                        }
                        0x2 => {
                            let rd = ((instruction >> 7) & 0b111) + 8;
                            let mut imm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                            imm = match (imm & 0x20) == 0 {
                                true => imm,
                                false => (0xc0 | imm) as i8 as i64 as u64,
                            };
                            self.c_andi(rd, imm)
                        }
                        0x3 => match ((instruction >> 12) & 0b1, (instruction >> 5) & 0b11) {
                            (0x0, 0x0) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_sub(rd, rs2)
                            }
                            (0x0, 0x1) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_xor(rd, rs2)
                            }
                            (0x0, 0x2) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_or(rd, rs2)
                            }
                            (0x0, 0x3) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_and(rd, rs2)
                            }
                            (0x1, 0x0) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_subw(rd, rs2)
                            }
                            (0x1, 0x1) => {
                                let rd = ((instruction >> 7) & 0b111) + 8;
                                let rs2 = ((instruction >> 2) & 0b111) + 8;
                                self.c_addw(rd, rs2)
                            }
                            (_, _) => panic!("invalid quadrant 2 funct2"),
                        },
                        _ => todo!(""),
                    }
                }
                0x5 => {
                    let mut offset = ((instruction >> 1) & 0x800) // offset[11]
                            | ((instruction << 2) & 0x400) // offset[10]
                            | ((instruction >> 1) & 0x300) // offset[9:8]
                            | ((instruction << 1) & 0x80) // offset[7]
                            | ((instruction >> 1) & 0x40) // offset[6]
                            | ((instruction << 3) & 0x20) // offset[5]
                            | ((instruction >> 7) & 0x10) // offset[4]
                            | ((instruction >> 2) & 0xe); // offset[3:1]

                    // Sign-extended.
                    offset = match (offset & 0x800) == 0 {
                        true => offset,
                        false => (0xf000 | offset) as i16 as i64 as u64,
                    };
                    self.c_j(offset)
                }
                0x6 => {
                    let rs1 = ((instruction >> 7) & 0b111) + 8;
                    // offset[8|4:3|7:6|2:1|5] = inst[12|11:10|6:5|4:3|2]
                    let mut offset = ((instruction >> 4) & 0x100) // offset[8]
                            | ((instruction << 1) & 0xc0) // offset[7:6]
                            | ((instruction << 3) & 0x20) // offset[5]
                            | ((instruction >> 7) & 0x18) // offset[4:3]
                            | ((instruction >> 2) & 0x6); // offset[2:1]
                                                          // Sign-extended.
                    offset = match (offset & 0x100) == 0 {
                        true => offset,
                        false => (0xfe00 | offset) as i16 as i64 as u64,
                    };
                    self.c_beqz(rs1, offset)
                }
                0x7 => {
                    let rs1 = ((instruction >> 7) & 0b111) + 8;
                    // offset[8|4:3|7:6|2:1|5] = inst[12|11:10|6:5|4:3|2]
                    let mut offset = ((instruction >> 4) & 0x100) // offset[8]
                            | ((instruction << 1) & 0xc0) // offset[7:6]
                            | ((instruction << 3) & 0x20) // offset[5]
                            | ((instruction >> 7) & 0x18) // offset[4:3]
                            | ((instruction >> 2) & 0x6); // offset[2:1]
                    offset = match (offset & 0x100) == 0 {
                        true => offset,
                        false => (0xfe00 | offset) as i16 as i64 as u64,
                    };
                    self.c_bnez(rs1, offset)
                }
                _ => todo!("quadrant 1 invalid funct3"),
            },
            // quadrant 2
            0b10 => match funct3 {
                0x0 => {
                    let rd = (instruction >> 7) & 0x1f;
                    let shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    self.c_slli(rd, shamt)
                }
                0x1 => {
                    let rd = (instruction >> 7) & 0x1f;
                    // offset[5|4:3|8:6] = inst[12|6:5|4:2]
                    let offset = ((instruction << 4) & 0x1c0) // offset[8:6]
                            | ((instruction >> 7) & 0x20) // offset[5]
                            | ((instruction >> 2) & 0x18); // offset[4:3]

                    self.c_fldsp(rd, offset)
                }
                0x2 => {
                    let rd = (instruction >> 7) & 0x1f;
                    let offset = ((instruction << 4) & 0xc0) // offset[7:6]
                            | ((instruction >> 7) & 0x20) // offset[5]
                            | ((instruction >> 2) & 0x1c); // offset[4:2]
                    self.c_lwsp(rd, offset)
                }
                0x3 => {
                    let rd = (instruction >> 7) & 0x1f;
                    // offset[5|4:3|8:6] = inst[12|6:5|4:2]
                    let offset = ((instruction << 4) & 0x1c0) // offset[8:6]
                            | ((instruction >> 7) & 0x20) // offset[5]
                            | ((instruction >> 2) & 0x18); // offset[4:3]
                    self.c_ldsp(rd, offset)
                }
                0x4 => match ((instruction >> 12) & 0x1, (instruction >> 2) & 0x1f) {
                    (0, 0) => {
                        let rs1 = (instruction >> 7) & 0x1f;
                        self.c_jr(rs1)
                    }
                    (0, _) => {
                        let rd = (instruction >> 7) & 0x1f;
                        let rs2 = (instruction >> 2) & 0x1f;
                        self.c_mv(rd, rs2)
                    }
                    (1, 0) => {
                        let rd = (instruction >> 7) & 0x1F;
                        if rd == 0 {
                            todo!("c.ebreak");
                        }
                        let rs1 = (instruction >> 7) & 0x1f;
                        self.c_jalr(rs1)
                    }
                    (1, _) => {
                        let rd = (instruction >> 7) & 0x1f;
                        let rs2 = (instruction >> 2) & 0x1f;
                        self.c_add(rd, rs2)
                    }
                    (_, _) => {
                        panic!("invalid quadrant 2 ")
                    }
                },
                0x5 => {
                    let rs2 = (instruction >> 2) & 0x1f;
                    // offset[5:3|8:6] = isnt[12:10|9:7]
                    let offset = ((instruction >> 1) & 0x1c0) // offset[8:6]
                            | ((instruction >> 7) & 0x38); // offset[5:3]
                    todo!("c.fsdsp")
                }
                0x6 => {
                    let rs2 = (instruction >> 2) & 0x1f;
                    // offset[5:2|7:6] = inst[12:9|8:7]
                    let offset = ((instruction >> 1) & 0xc0) // offset[7:6]
                            | ((instruction >> 7) & 0x3c); // offset[5:2]
                    self.c_swsp(rs2, offset)
                }
                0x7 => {
                    let rs2 = (instruction >> 2) & 0x1f;
                    let offset = ((instruction >> 1) & 0x1c0) // offset[8:6]
                            | ((instruction >> 7) & 0x38); // offset[5:3]
                    self.c_sdsp(rs2, offset)
                }
                _ => todo!("quadrant 2 invalid funct3"),
            },
            _ => todo!("invalid opcdoe"),
        }
    }
    // EOF
    // execute instructioon
    pub fn execute(self: &mut Self, instruction: u64) -> bool {
        let opcode = instruction & 0x0000007f;
        let rd = (instruction & 0x00000f80) >> 7;
        let rs1 = (instruction & 0x000f8000) >> 15;
        let rs2 = (instruction & 0x01f00000) >> 20;
        let funct3 = (instruction & 0x00007000) >> 12;
        let funct7 = (instruction & 0xfe000000) >> 25;
        let funct6 = funct7 >> 1;
        let funct5 = (funct7 & 0b1111100) >> 2;
        let shamt = (instruction >> 20) & 0x3f;

        let imm= ((instruction as i32 as i64) >> 20) as u64;
        let imm_s_type = (((instruction & 0xfe000000) as i32 as i64 >> 20) as u64) | ((instruction >> 7) & 0x1f);
        let imm_b_type = (((instruction & 0x80000000) as i32 as i64 >> 19) as u64)
        | ((instruction & 0x80) << 4) // imm[11]
        | ((instruction >> 20) & 0x7e0) // imm[10:5]
        | ((instruction>> 7) & 0x1e); // imm[4:1]

        let imm_j_type = (((instruction & 0x80000000) as i32 as i64 >> 11) as u64) // imm[20]
        | (instruction & 0xff000) // imm[19:12]
        | ((instruction >> 9) & 0x800) // imm[11]
        | ((instruction >> 20) & 0x7fe); // imm[10:1]

        match opcode {
            0b1110011 => match funct3 {
                0x0 => match funct7 {
                    0x0 => self.ecall(),
                    0x1 => self.ebreak(),
                    // .. other instructions uret, wfi sret
                    _ => panic!("invalid funct7"),
                },
                // CSR INSTRUCTIONS
                /*
                0x1 => self.csrrw(csr, rs1, rd),
                0x2 => self.csrrs(csr, rs1, rd),
                0x3 => self.csrrc(csr, rs1, rd),
                0x5 => self.csrrwi(csr, rs1, rd),
                0x6 => self.csrrsi(csr, rs1, rd),
                0x7 => self.csrrci(csr, rs1, rd),
                */
                _ => todo!("Support CSR Instructions"),
            },
            0b0101111 => match funct3 {
                0x3 => match funct5 {
                    /*
                    0x2 => self.load_double_word_atomic(rd, rs1),
                    0x3 => self.store_double_word_atomic(rd, rs1, rs2),
                    0x1 => self.swap_double_word_atomic(rd, rs1, rs2),
                    0x0 => self.add_double_word_atomic(rd, rs1, rs2),
                    0xC => self.and_double_word_atomic(rd, rs1, rs2),
                    0x8 => self.or_double_word_atomic(rd, rs1, rs2),
                    0x4 => self.xor_double_word_atomic(rd, rs1, rs2),
                    0x14 => self.max_double_word_atomic(rd, rs1, rs2),
                    0x10 => self.min_double_word_atomic(rd, rs1, rs2),
                    0x18 => self.minu_double_word_atomic(rd, rs1, rs2),
                    0x1C => self.maxu_double_word_atomic(rd, rs1, rs2),
                    */
                    _ => todo!("Support Atomic Instructions"),
                },
                0x2 => match funct5 {
                    /* 
                    0x2 => self.load_word_atomic(rd, rs1),
                    0x3 => self.store_word_atomic(rd, rs1, rs2),
                    0x1 => self.swap_word_atomic(rd, rs1, rs2),
                    0x0 => self.add_word_atomic(rd, rs1, rs2),
                    0xC => self.and_word_atomic(rd, rs1, rs2),
                    0x8 => self.or_word_atomic(rd, rs1, rs2),
                    0x4 => self.xor_word_atomic(rd, rs1, rs2),
                    0x14 => self.max_word_atomic(rd, rs1, rs2),
                    0x10 => self.min_word_atomic(rd, rs1, rs2),
                    0x18 => self.minu_word_atomic(rd, rs1, rs2),
                    0x1C => self.maxu_word_atomic(rd, rs1, rs2),
                    */
                _ => todo!("Support Atomic Instructions"),
                },
                _ => todo!("Support Atomic Instructions"),
            },
            0b0110011 => match funct3 {
                0x0 => match funct7 {
                    0x0 => self.add(rd, rs1, rs2),
                    0x1 => self.mul(rd, rs1, rs2),
                    0x20 => self.sub(rd, rs1, rs2),
                    _ => panic!("Invalid funct7"),
                },
                0x5 => match funct7 {
                    0x0 => self.srl(rd, rs1, rs2),
                    0x1 => self.divu(rd, rs1, rs2),
                    0x20 => self.sra(rd, rs1, rs2),
                    _ => panic!("Invalid funct7"),
                },
                0x4 => match funct7 {
                    0x0 => self.xor(rd, rs1, rs2),
                    0x1 => self.div(rd, rs1, rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x6 => match funct7 {
                    0x0 => self.or(rd, rs1, rs2),
                    0x1 => self.rem(rd, rs1, rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x7 => match funct7 {
                    0x0 => self.and(rd, rs1, rs2),
                    0x1 => self.remu(rd, rs1, rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x1 => match funct7 {
                    0x0 => self.sll(rd, rs1, rs2),
                    0x1 => self.mulh(rd, rs1, rs2),
                    _ => panic!("INVALID FUNCT7"),
                },
                0x2 => match funct7 {
                    0x0 => self.slt(rd, rs1, rs2),
                    0x1 => self.mulhsu(rd, rs1, rs2),
                    _ => panic!("invalid funct 7"),
                },
                0x3 => match funct7 {
                    0x0 => self.sltu(rd, rs1, rs2),
                    0x1 => self.mulu(rd, rs1, rs2),
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
                0x0 => self.store_byte(rs2, rs1, imm_s_type),
                0x1 => self.store_half(rs2, rs1, imm_s_type),
                0x2 => self.store_word(rs2, rs1, imm_s_type),
                0x3 => self.store_double_word(rs2, rs1, imm_s_type),
                _ => panic!("Invalid funct3 {:#08X}", funct3),
            },
            0b0010011 => match funct3 {
                // I TYPE
                0x0 => self.addi(rd, rs1, imm),
                0x1 => self.slli(rd, rs1, shamt),
                0x2 => self.slti(rd, rs1, imm),
                0x3 => self.sltiu(rd, rs1, imm),
                0x4 => self.xori(rd, rs1, imm),
                0x5 => match funct6 {
                    0x0 => self.srli(rd, rs1, shamt),
                    0x10 => self.srai(rd, rs1, shamt),
                    _ => panic!("INVALID IMMEDIATE 5-11"),
                },
                0x6 => self.ori(rd, rs1, imm),
                0x7 => self.andi(rd, rs1, imm),
                _ => panic!("Unimplemented funct3"),
            },
            0b1100011 => match funct3 {
                0x0 => self.beq(rs1, rs2, imm_b_type),
                0x1 => self.bne(rs1, rs2, imm_b_type),
                0x4 => self.blt(rs1, rs2, imm_b_type),
                0x5 => self.bge(rs1, rs2, imm_b_type),
                0x6 => self.bltu(rs1, rs2, imm_b_type),
                0x7 => self.bgeu(rs1, rs2, imm_b_type),
                _ => panic!("PANIC INVAID OPCODE"),
            },
            0b1101111 => self.jal(rd, imm_j_type),
            0b1100111 => self.jalr(rd, rs1, imm),
            0b0110111 => self.lui(rd, (instruction & 0xfffff000) as i32 as i64 as u64),
            0b0010111 => self.auipc(rd, (instruction & 0xfffff000) as i32 as i64 as u64),
            0b0011011 => match funct3 {
                0x0 => self.addiw(rd, rs1, imm),
                0x1 => self.slliw(rd, rs1, shamt),
                0x3 => match funct7 {
                    0x0 => self.srliw(rd, rs1, shamt),
                    0x20 => self.sraiw(rd, rs1, imm),
                    _ => panic!("invalid funct7"),
                },
                _ => panic!("invalid funct"),
            },
            0b0111011 => match funct3 {
                0x7 => match funct7 {
                    0x1 => self.remuw(rd, rs1, rs2),
                    _ => panic!(""),
                },
                0x6 => match funct7 {
                    0x1 => self.remw(rd, rs1, rs2),
                    _ => panic!(""),
                },
                0x4 => match funct7 {
                    0x1 => self.divw(rd, rs1, rs2),
                    _ => panic!(""),
                },
                0x0 => match funct7 {
                    0x0 => self.addw(rd, rs1, rs2),
                    0x20 => self.subw(rd, rs1, rs2),
                    0x1 => self.mulw(rd, rs1, rs2),
                    _ => panic!("invalid funct7"),
                },
                0x1 => self.sllw(rd, rs1, rs2),
                0x5 => match funct7 {
                    0x1 => self.divuw(rd, rs1, rs2),
                    0x0 => self.srlw(rd, rs1, rs2),
                    0x20 => self.sraw(rd, rs1, rs2),
                    _ => panic!("invalid funct7"),
                },
                _ => panic!("unknown funct3"),
            },
            _ => panic!("PC: {:#08X} Unimplemented OpCode {:#013b}", self.pc, opcode),
        }
        // match on opcode then match on func3?
    }

    fn remu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
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

    fn rem(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
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

    fn divu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
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

    fn div(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
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
    fn c_slli(&mut self, rd: u64, shamt: u64) -> bool {
        println!("{:#08X} c.slli x{rd},x{rd},{:#04X}", self.pc, shamt);
        self.x_reg[rd as usize] = self.x_reg[rd as usize] << shamt;
        false
    }
    fn c_fldsp(&mut self, rd: u64, offset: u64) -> bool {
        todo!("floatingpoint");
        false
    }
    fn c_sdsp(&mut self, rs2: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.sdsp x{rs2},{offset}(sp)", self.pc);
        }
        let _memory_address = self.x_reg[2].wrapping_add(offset as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u64;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn c_swsp(&mut self, rs2: u64, offset: u64) -> bool {
        println!("swsp");
        let _memory_address = self.x_reg[2].wrapping_add(offset as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u64;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn c_lwsp(&mut self, rd: u64, offset: u64) -> bool {
        println!("lwsp");
        let value = self.x_reg[2].wrapping_add(offset as u64) as i32 as i64 as u64;
        self.x_reg[rd as usize] = value;
        false
    }
    fn c_ldsp(&mut self, rd: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("c.ldsp x{rd} {offset},(x2)");
        }
        let _memory_address = self.x_reg[2].wrapping_add(offset);
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn c_add4spn(&mut self, rd: u64, nzuimm: u64) -> bool {
        println!("c.addi4spn");
        let temp = self.x_reg[2].wrapping_add(nzuimm as u64);
        self.x_reg[rd as usize] = temp;
        false
    }
    fn c_fld(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        todo!("FLOATING POINT REGISTERS");
        false
    }
    fn c_lw(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.lw x{rd},{offset},(x{rs1})", self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        if self.debug_flag {
            println!("Memory Adress {:#08X}", _memory_address);
            println!("Dereferenced Memory Adress {:#08X}", _memory_address);
            println!("{:#08X}", self.mmu.virtual_memory[0x00FFFF])
        }
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i64 as u64;
        if self.debug_flag {
            println!("Memory Adress {:#08X}", _memory_address);
            println!("Dereferenced Memory Adress {:#08X}", result);
        }
        self.x_reg[rd as usize] = result as i32 as i64 as u64;
        false
    }
    fn c_ld(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.ld x{rd},{offset},(x{rs1})", self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        if self.debug_flag {
            println!("{:#08X} {:#08X}", _memory_address, offset);
        }
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result as i32 as i64 as u64;
        false
    }
    fn c_fsd(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        todo!("floating point");
        false
    }

    fn c_sw(&mut self, rs2: u64, rs1: u64, offset: u64) -> bool {
        println!("c_sw");
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn c_sd(&mut self, rs2: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.sd x{rs2},{}(x{rs1})", self.pc, offset as i16);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u64;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn c_addi(&mut self, rd: u64, nzimm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.addi x{rd},x{rd},{}", self.pc, nzimm as i16);
        }
        if rd != 0 {
            self.x_reg[rd as usize] = self.x_reg[rd as usize].wrapping_add(nzimm as u64);
        }
        false
    }
    fn c_addiw(&mut self, rd: u64, nzimm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.addiw x{rd},x{rd},{}", self.pc, nzimm as i16);
        }
        if rd != 0 {
            self.x_reg[rd as usize] =
                self.x_reg[rd as usize].wrapping_add(nzimm as u64) as i32 as i64 as u64;
        }
        false
    }

    fn c_lui(&mut self, rd: u64, nzimm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.lui x{rd},{:#08X}", self.pc, nzimm);
        }
        if nzimm != 0 {
            self.x_reg[rd as usize] = nzimm as u64;
        }
        false
    }

    fn c_li(&mut self, rd: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.li x{rd},{:#08X}", self.pc, imm);
        }
        if rd != 0 {
            self.x_reg[rd as usize] = imm as u64;
        }
        false
    }

    fn c_addi16sp(&mut self, rd: u64, nzimm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.addi16sp sp,sp,{}", self.pc, nzimm as i16);
        }
        if nzimm != 0 {
            self.x_reg[2] = self.x_reg[2].wrapping_add(nzimm as u64);
        }
        false
    }
    fn c_srli(&mut self, rd: u64, shamt: u64) -> bool {
        println!("srli");
        self.x_reg[rd as usize] = self.x_reg[rd as usize] >> shamt;
        false
    }
    fn c_srai(&mut self, rd: u64, shamt: u64) -> bool {
        println!("srai");
        self.x_reg[rd as usize] = ((self.x_reg[rd as usize] as i64) >> shamt) as u64;
        false
    }

    fn c_andi(&mut self, rd: u64, imm: u64) -> bool {
        println!("andi");
        self.x_reg[rd as usize] = self.x_reg[rd as usize] & imm as u64;
        false
    }

    fn c_sub(&mut self, rd: u64, rs2: u64) -> bool {
        println!("sub");
        self.x_reg[rd as usize] = self.x_reg[rd as usize].wrapping_sub(self.x_reg[rs2 as usize]);
        false
    }

    fn c_xor(&mut self, rd: u64, rs2: u64) -> bool {
        println!("xor");
        self.x_reg[rd as usize] = self.x_reg[rd as usize] ^ self.x_reg[rs2 as usize];
        false
    }
    fn c_or(&mut self, rd: u64, rs2: u64) -> bool {
        println!("or");
        self.x_reg[rd as usize] = self.x_reg[rd as usize] ^ self.x_reg[rs2 as usize];
        false
    }
    fn c_and(&mut self, rd: u64, rs2: u64) -> bool {
        println!("and");
        self.x_reg[rd as usize] = self.x_reg[rd as usize] & self.x_reg[rs2 as usize];
        false
    }

    fn c_subw(&mut self, rd: u64, rs2: u64) -> bool {
        println!("subw");
        self.x_reg[rd as usize] =
            self.x_reg[rd as usize].wrapping_sub(self.x_reg[rs2 as usize]) as i32 as i64 as u64;
        false
    }
    fn c_add(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("c.add x{rd},x{rd},x{rs2}");
        }
        if rs2 != 0 {
            self.x_reg[rd as usize] =
                self.x_reg[rd as usize].wrapping_add(self.x_reg[rs2 as usize]);
        }
        false
    }

    fn c_addw(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("c.addw x{rd},x{rd},x{rs2}");
        }
        self.x_reg[rd as usize] =
            self.x_reg[rd as usize].wrapping_add(self.x_reg[rs2 as usize]) as i32 as i64 as u64;
        false
    }

    fn mulu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("MULU");
        let left = self.x_reg[rs1 as usize] as u64 as u128;
        let right = self.x_reg[rs2 as usize] as u64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn sll(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} sll",self.pc);
        }
        let shamt = self.x_reg[rs2 as usize] & 0x3f;
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] << shamt;
        false
    }

    fn slt(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} slt",self.pc);
        }
        if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[rs2 as usize] as i64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn slli(&mut self, rd: u64, rs1: u64, shamt: u64) -> bool {
        println!("SLLI");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] << shamt;
        false
    }

    fn srli(&mut self, rd: u64, rs1: u64, shamt: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} srli",self.pc);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] >> shamt;
        false
    }

    fn srai(&mut self, rd: u64, rs1: u64, shamt: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} srai",self.pc);
        }
        self.x_reg[rd as usize] = ((self.x_reg[rs1 as usize] as i64) >> shamt) as u64;
        false
    }

    fn slti(&mut self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("SRLTI");
        if (self.x_reg[rs1 as usize] as i64) < (imm as i64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltiu(&mut self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("SLTIU");
        if (self.x_reg[rs1 as usize] as u64) < imm {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SLTU");
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn mulhsu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("MULHSU");
        let left = self.x_reg[rs1 as usize] as i64 as u128;
        let right = self.x_reg[rs2 as usize] as u64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn srl(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SRL");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] >> (self.x_reg[rs2 as usize] & 0b11111);
        false
    }

    fn sra(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SRA");
        self.x_reg[rd as usize] =
            self.x_reg[rs1 as usize] >> ((self.x_reg[rs2 as usize] & 0b11111) as i64) as u64;
        false
    }

    fn mulh(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} mulh",self.pc);
        }
        let left = self.x_reg[rs1 as usize] as i64 as u128;
        let right = self.x_reg[rs2 as usize] as i64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn mul(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} mul",self.pc);
        }
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as i64).wrapping_mul(self.x_reg[rs2 as usize] as i64) as u64;
        false
    }

    fn sub(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} sub",self.pc);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_sub(self.x_reg[rs2 as usize]);
        false
    }

    fn add(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!(
                "{:#08X} add x{rd} ({:#08X}) x{rs1} ({:#08X}) x{rs2} ({:#08X})",
                self.pc,
                self.x_reg[rd as usize],
                self.x_reg[rs1 as usize],
                self.x_reg[rs2 as usize]
            );
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_add(self.x_reg[rs2 as usize]);
        false
    }
    fn store_double_word(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        println!("SD RS1 = {:#08X}", self.x_reg[rs1 as usize]);
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u64;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_word(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        println!("SW");
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_half(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        println!("SH");
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u16;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 2]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn store_byte(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let index = rs2 as usize;
        let value = self.x_reg[index] as u8;
        if self.debug_flag {
            println!(
                "{:#08X} SB {:#08X} <- {:#08X}",
                self.pc, _memory_address, value
            )
        }
        self.mmu.virtual_memory[_memory_address as usize] = value;
        false
    }
    fn load_word_unsigned(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} lwu",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]);
        self.x_reg[rd as usize] = result as u64;
        false
    }
    fn load_double_word(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("LD");
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        println!("{:#08X}", _memory_address);
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        self.x_reg[rd as usize] = result;
        false
    }
    fn load_double_word_atomic(self: &mut Self, rd: u64, rs1: u64) -> bool {
        println!("LR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    fn store_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SC.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 8]
            .copy_from_slice(&value_as_bytes);
        self.x_reg[rd as usize] = 0;
        false
    }
    fn load_word_atomic(self: &mut Self, rd: u64, rs1: u64) -> bool {
        println!("LR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i64 as u64;
        self.x_reg[rd as usize] = result as i64 as u64;
        false
    }
    fn store_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SC.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        self.x_reg[rd as usize] = 0;
        false
    }

    fn maxu_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAXU.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as u32 as u64;
        let min = if result as u64 > self.x_reg[rs2 as usize] {
            self.x_reg[rs2 as usize]
        } else {
            result
        };
        self.x_reg[rd as usize] = min;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn minu_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMINU.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as u32 as u64;
        let min = if result as u64 > self.x_reg[rs2 as usize] {
            self.x_reg[rs2 as usize]
        } else {
            result
        };
        self.x_reg[rd as usize] = min;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn min_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMIN.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let min = if result > self.x_reg[rs2 as usize] as i32 {
            self.x_reg[rs2 as usize] as i32
        } else {
            result
        };
        self.x_reg[rd as usize] = min as u64;
        let value_as_bytes = min.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn max_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAX.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let max = if result > self.x_reg[rs2 as usize] as i32 {
            result
        } else {
            self.x_reg[rs2 as usize] as i32
        };
        self.x_reg[rd as usize] = max as u64;
        let value_as_bytes = max.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn xor_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOXOR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 ^ self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn or_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOOR.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 | self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn and_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOAND.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 & self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn add_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOADD.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        let newrd = result as u64 + self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn minu_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMINU.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
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
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn maxu_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAXU.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
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
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn max_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAX.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
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
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn min_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMIN.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
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
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn xor_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOXOR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 ^ self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn and_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOAND.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 & self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn or_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOOR.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 | self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }
    fn add_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOADD.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
        let result = u64::from_le_bytes([
            value0, value1, value2, value3, value4, value5, value6, value7,
        ]);
        let newrd = result as u64 + self.x_reg[rs2 as usize];
        self.x_reg[rd as usize] = newrd;
        // write to the memory address now
        let value_as_bytes = newrd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn swap_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOSWAP.D");
        let _memory_address = self.x_reg[rs1 as usize];
        let value0 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value1 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 4] as u8;
        let value5 = self.mmu.virtual_memory[_memory_address as usize + 5] as u8;
        let value6 = self.mmu.virtual_memory[_memory_address as usize + 6] as u8;
        let value7 = self.mmu.virtual_memory[_memory_address as usize + 7] as u8;
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
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn swap_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOSWAP.W");
        let _memory_address = self.x_reg[rs1 as usize];
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32;
        self.x_reg[rd as usize] = result as u64;
        let oldrs2 = self.x_reg[rs2 as usize];
        self.x_reg[rs2 as usize] = result as u64;
        self.x_reg[rd as usize] = oldrs2;
        let swapped_rd = self.x_reg[rd as usize];
        // write to the memory address now
        let value_as_bytes = swapped_rd.to_le_bytes();
        self.mmu.virtual_memory[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
        false
    }

    fn load_word(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
        println!("{:#08X} lw",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.virtual_memory[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.virtual_memory[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i32 as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 16 bit value
    fn load_half(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("lh");
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]) as i16 as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 8 bit value
    fn load_byte(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value = self.mmu.virtual_memory[_memory_address as usize] as i8;
        if self.debug_flag {
            println!(
                "{:#08X} LB x{rd} ({:#08X}) {:#08X} -> ({:#08X})",
                self.pc, self.x_reg[rd as usize], _memory_address, value
            );
        }

        self.x_reg[rd as usize] = value as u64;
        false
    }
    // load 16 bit value
    fn load_half_u(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("lhu");
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.virtual_memory[_memory_address as usize] as u8;
        let value2 = self.mmu.virtual_memory[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]);
        self.x_reg[rd as usize] = result as u64;
        false
    }
    // load 8 bit value
    fn load_byte_u(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} lbu",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let value = self.mmu.virtual_memory[_memory_address as usize];
        self.x_reg[rd as usize] = value as u64;
        false
    }
    // add immediate
    fn addi(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} addi x{rd}, x{rs1}, {}", self.pc, imm);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_add(imm);
        false
    }
    fn andi(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} andi x{rd}, x{rs1}, {}", self.pc, imm as i64);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & imm;
        false
    }
    fn ori(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} ori x{rd}, x{rs1}, {}", self.pc, imm);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | imm;
        false
    }
    fn xori(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} xori x{rd}, x{rs1}, {}", self.pc, imm);
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ imm;
        false
    }
    fn and(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AND");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & self.x_reg[rs2 as usize];
        false
    }
    fn or(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("OR");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | self.x_reg[rs2 as usize];
        false
    }
    fn xor(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("XOR");
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ self.x_reg[rs2 as usize];
        false
    }

    fn bgeu(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        println!("BGEU");
        if (self.x_reg[rs1 as usize] as u64) >= (self.x_reg[rs2 as usize] as u64) {
            self.pc = self.pc.wrapping_add(imm_b_type as i64 as u64);
            return true;
        }
        false
    }
    fn bltu(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        println!("bltu");
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.pc = self.pc.wrapping_add(imm_b_type as i64 as u64);
            return true;
        }
        false
    }

    fn bge(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        println!("bge");
        if (self.x_reg[rs1 as usize] as i64) >= (self.x_reg[rs2 as usize] as i64) {
            self.pc = self.pc.wrapping_add(imm_b_type as u64);
            return true;
        }
        false
    }

    fn blt(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        if self.debug_flag {
            println!("blt x{rs1},x{rs2},{:#08X}", imm_b_type);
            println!(
                "if {:#016X} less than {:#08X}",
                self.x_reg[rs1 as usize] as i64, self.x_reg[rs2 as usize] as i64
            );
        }
        if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[rs2 as usize] as i64) {
            self.pc = self.pc.wrapping_add(imm_b_type);
            return true;
        }
        false
    }

    fn bne(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        println!("bne");
        if self.x_reg[rs1 as usize] != self.x_reg[rs2 as usize] {
            self.pc = self.pc.wrapping_add(imm_b_type as u64);
            return true;
        }
        false
    }
    fn beq(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        if self.debug_flag {
            println!(
                "beq if x{rs2} == x{rs1} -> x{:#08X}",
                self.pc.wrapping_add(imm_b_type as u64)
            );
        }
        if self.x_reg[rs1 as usize] == self.x_reg[rs2 as usize] {
            self.pc = self.pc.wrapping_add(imm_b_type as u64);
            return true;
        }
        false
    }
    fn jal(self: &mut Self, rd: u64, imm_j_type: u64) -> bool {
        println!("JAL");
        self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
        self.pc = self.pc.wrapping_add(imm_j_type as i64 as u64);
        true
    }
    fn c_beqz(self: &mut Self, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("c.beqz x{rs1} {:#08X}", offset);
        }
        if self.x_reg[rs1 as usize] == 0 {
            self.pc = self.pc.wrapping_add(offset as u64);
            return true;
        }
        false
    }
    fn c_bnez(self: &mut Self, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("c.bnez x{rs1},x0,{:#08X}", offset);
        }
        if self.x_reg[rs1 as usize] != 0 {
            self.pc = self.pc.wrapping_add(offset as u64);
            return true;
        }
        false
    }

    fn c_mv(self: &mut Self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.mv x{rd}, x{rs2}", self.pc);
        }
        if rs2 != 0 {
            self.x_reg[rd as usize] = self.x_reg[rs2 as usize];
        }
        false
    }
    fn c_j(self: &mut Self, offset: u64) -> bool {
        if self.debug_flag {
            println!(
                "{:#08X} c.j {:#08X}",
                self.pc,
                self.pc.wrapping_add(offset as u64)
            );
        }
        self.pc = self.pc.wrapping_add(offset as u64);
        true
    }
    fn c_jr(self: &mut Self, rs1: u64) -> bool {
        if self.debug_flag {
            println!("c.JR");
        }
        if rs1 != 0 {
            self.pc = self.x_reg[rs1 as usize];
            return true;
        }
        false
    }
    fn c_jalr(self: &mut Self, rs1: u64) -> bool {
        println!("c.JALR");
        let t = self.pc + 2;
        self.pc = self.x_reg[rs1 as usize];
        self.x_reg[1] = t; // set x0 to ret
        true
    }
    fn jalr(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        let t = self.pc.wrapping_add(4);
        let target = ((self.x_reg[rs1 as usize] as i64).wrapping_add(imm as i64)) & !1;
        self.pc = target as u64;
        self.x_reg[rd as usize] = t;
        if self.debug_flag {
            println!("JALR {}(x{rd} PC {:#08X})", imm as i64, self.pc);
        }
        true
    }
    fn lui(self: &mut Self, rd: u64, imm_u_type: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} LUI x{},{:#08X}", self.pc, rd, imm_u_type);
        }
        self.x_reg[rd as usize] = imm_u_type;
        false
    }
    fn auipc(self: &mut Self, rd: u64, imm_u_type: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} AUIPC x{} {:#08X}", self.pc, rd, imm_u_type);
        }
        self.x_reg[rd as usize] = self.pc.wrapping_add(imm_u_type);
        false
    }

    fn addiw(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        println!("ADDIW confirm works");
        let rs1__ = self.x_reg[rs1 as usize];
        self.x_reg[rd as usize] = rs1__.wrapping_add(imm) as i64 as u64;
        false
    }

    fn slliw(self: &mut Self, rd: u64, rs1: u64, shamt: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} slliw",self.pc);
        }
        let shamt =  shamt as u32;
        self.x_reg[rd as usize] = (left >> right) as u32 as i64 as u64;
        false
    }

    fn srliw(self: &mut Self, rd: u64, rs1: u64, shamt: u64) -> bool {
        println!("SRLIW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = shamt as u32;
        self.x_reg[rd as usize] = (left << right) as u32 as i64 as u64;
        false
    }

    fn sraw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} sraw",self.pc);
        }
        let left = self.x_reg[rs1 as usize] as i32;
        let shamt = self.x_reg[rs2 as usize] & 0x1f;
        self.x_reg[rd as usize] = (left >> shamt ) as i64 as u64;
        false
    }
    fn sraiw(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} sraiw", self.pc);
        }
        let left = self.x_reg[rs1 as usize] as i32;
        let shamt = (imm & 0x1f) as u32;
        self.x_reg[rd as usize] = (left >> shamt)  as i64 as u64;
        false
    }
    fn srlw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SRLW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32 & 0b11111;
        self.x_reg[rd as usize] = (left >> right) as u32 as u64;
        false
    }
    fn sllw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SLLW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32 & 0b11111;
        self.x_reg[rd as usize] = (left << right) as u32 as u64;
        false
    }

    fn divuw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
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

    fn mulw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("MULW");
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32;
        let result = left.wrapping_mul(right);
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }
    fn subw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SUBW");
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_sub(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }
    fn addw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("ADDW");
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_add(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }

    fn divw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
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
    fn remw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
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
    fn remuw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
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

    fn csrrw(&mut self, csr: u64, rs1: u64, rd: u64) -> bool {
        println!("CSRRW");
        if rd == 0 {
            return false;
        }
        let oldcsr = self.csr_reg[csr as usize] as u64;
        let oldrs1 = self.csr_reg[rs1 as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = oldrs1;
        false
    }

    fn csrrwi(&mut self, csr: u64, imm: u64, rd: u64) -> bool {
        println!("CSRRWI");
        let oldcsr = self.csr_reg[csr as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = imm as u64;
        false
    }

    fn csrrs(&mut self, csr: u64, rs1: u64, rd: u64) -> bool {
        println!("CSRRS");
        let oldcsr = self.csr_reg[csr as usize] as u64;
        let mask = self.csr_reg[rs1 as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = self.csr_reg[csr as usize] | mask;
        false
    }

    fn csrrc(&mut self, csr: u64, rs1: u64, rd: u64) -> bool {
        println!("CSRRC");
        let oldcsr = self.csr_reg[csr as usize] as u64;
        let mask = self.csr_reg[rs1 as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = self.csr_reg[csr as usize] & !mask;
        false
    }

    fn csrrsi(&mut self, csr: u64, imm: u64, rd: u64) -> bool {
        println!("CSRRSI");
        let oldcsr = self.csr_reg[csr as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = self.csr_reg[csr as usize] | imm as u64;
        false
    }

    fn csrrci(&mut self, csr: u64, imm: u64, rd: u64) -> bool {
        println!("CSRRCI");
        let oldcsr = self.csr_reg[csr as usize] as u64;
        self.x_reg[rd as usize] = oldcsr;
        self.csr_reg[csr as usize] = self.csr_reg[csr as usize] & !imm as u64;
        false
    }

    fn ebreak(&self) -> bool {
        loop {
            println!("DEBUG BREAK");
            break;
        }
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
        println!("syscall {:#08X}", syscall);
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        match syscall {
            0x62 => {
                // https://man7.org/linux/man-pages/man2/lseek.2.html
                let fd = _a0;
                let offset = _a1;
                let whence = _a2;
                println!("{:#08X} {:#08X} {:#08X}", fd, offset, whence);
                self.x_reg[10] = fd;
                todo!("lseek");
            }
            0x60 => {
                // https://man7.org/linux/man-pages/man2/set_tid_address.2.html
                let tidptr = _a0;
                // return value set current pid as value
                self.x_reg[10] = std::process::id() as u64;
            }
            0x40 => {
                let fd = _a0;
                let end = _a1 + _a2;
                let raw_bytes = &self.mmu.virtual_memory[_a1 as usize..end as usize];
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
                panic!("Unimplemented syscall -> {:#08X}", syscall);
            }
        }
        false
    }
}
