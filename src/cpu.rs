use crate::data;
use crate::mmu::MMU;
use crate::data::Iovec;
use std::{collections::HashMap, os::fd::AsRawFd};



#[derive(Debug,Clone)]
pub struct CPU {
    pub pc: u64,
    pub sp: u64,
    pub call_stack:Vec<u64>,
    pub mmu: MMU,
    pub x_reg: [u64; 32],
    pub f_reg: [u64; 32],
    pub csr_reg: [u64; 4096],
    pub file_descriptors: HashMap<u64,std::os::fd::RawFd>,
    pub current_compressed: bool,
    pub was_last_compressed: bool,
    pub debug_flag: bool,
    pub exit_called:bool,
    pub exit_status:i32
}

// XLEN = u64 arch size
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
        display_string.push_str("TODO FIX CALL STACK\n");
        /*
        for (i,item) in self.call_stack.iter().enumerate() {
            let s = format!("{i} {:#08X}\n",item);
            display_string.push_str(&s);
        }
        */
        write!(f, "{}", display_string)
    }
}

impl CPU {
    pub fn new() -> Self {
        let mmu = MMU::new();
        let mut fds:[i32;1024] = [-1;1024];
        fds[0] = 0;
        fds[1] = 1;
        fds[2] = 2;
        let xreg: [u64; 32] = [0; 32];
        CPU {
            sp: 0,
            pc: 0x00000000,
            call_stack:vec![0;0],
            mmu: mmu,
            x_reg: xreg,
            f_reg: [0; 32],
            csr_reg: [0; 4096],
            file_descriptors: HashMap::new(),
            current_compressed: false,
            was_last_compressed: false,
            debug_flag: true,
            exit_status:0,
            exit_called: false,
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
                0x4 => self.load_byte_u(rd, rs1, imm), // todo write test
                0x5 => self.load_half_u(rd, rs1, imm), // todo write test
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
                0x1 => self.slliw(rd, rs1, imm),
                0x5 => match funct7 {
                    0x0 => self.srliw(rd, rs1, shamt),
                    0x20 => self.sraiw(rd, rs1, imm),
                    _ => panic!("invalid funct7"),
                },
                _ => panic!("invalid funct {funct3}"),
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
        if self.debug_flag {
            println!("REMU");
        }
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
        if self.debug_flag {
            println!("REM");
        }
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
        if self.debug_flag {
            println!("DIVU");
        }
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
        if self.debug_flag {
            println!("DIV");
        }
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
        if self.debug_flag{
            println!("{:#08X} c.slli x{rd},x{rd},{:#04X}", self.pc, shamt);
        }
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
        let value = self.x_reg[rs2 as usize];
        self.mmu.write_double_word(_memory_address, value);
        false
    }

    fn c_swsp(&mut self, rs2: u64, offset: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} swsp",self.pc);
        }
        let _memory_address = self.x_reg[2].wrapping_add(offset);
        let value = self.x_reg[rs2 as usize];
        self.mmu.write_word(_memory_address, value);
        false
    }
    fn c_lwsp(&mut self, rd: u64, offset: u64) -> bool {
        if self.debug_flag{println!("lwsp");}
        let address = self.x_reg[2].wrapping_add(offset);
        let result = self.mmu.read_word(address);
        self.x_reg[rd as usize] = result as i32 as i64 as u64;
        false
    }
    fn c_ldsp(&mut self, rd: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("c.ldsp x{rd} {offset},(x2)");
        }
        let address = self.x_reg[2].wrapping_add(offset);
        let result = self.mmu.read_double_word(address);
        self.x_reg[rd as usize] = result;
        false
    }

    fn c_add4spn(&mut self, rd: u64, nzuimm: u64) -> bool {
        if self.debug_flag{println!("c.addi4spn");}
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
        //let result = u32::from_le_bytes(self.mmu.read(_memory_address, WORD).try_into().unwrap());
        let result = self.mmu.read_word(_memory_address);
        self.x_reg[rd as usize] = result as i32 as i64 as u64;
        false
    }
    fn c_ld(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} c.ld x{rd},{offset},(x{rs1})", self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        //let result = u64::from_le_bytes(self.mmu.read(_memory_address, DOUBLE_WORD).try_into().unwrap());
        let result = self.mmu.read_double_word(_memory_address);
        self.x_reg[rd as usize] = result;
        false
    }
    fn c_fsd(&mut self, rd: u64, rs1: u64, offset: u64) -> bool {
        todo!("floating point");
        false
    }

    fn c_sw(&mut self, rs2: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} c_sw x{rs2}{offset}(x{rs1})",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        let value = self.x_reg[rs2 as usize];
        self.mmu.write_word(_memory_address, value);
        false
    }
    fn c_sd(&mut self, rs2: u64, rs1: u64, offset: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} c_sd x{rs2}{offset}(x{rs1})",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(offset as u64);
        let value = self.x_reg[rs2 as usize];
        //self.mmu.write(_memory_address, value, DOUBLE_WORD);
        self.mmu.write_double_word(_memory_address, value);
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
        if self.debug_flag{println!("srli");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize] >> shamt;
        false
    }
    fn c_srai(&mut self, rd: u64, shamt: u64) -> bool {
        if self.debug_flag{println!("srai");}
        self.x_reg[rd as usize] = ((self.x_reg[rd as usize] as i64) >> shamt) as u64;
        false
    }

    fn c_andi(&mut self, rd: u64, imm: u64) -> bool {
        if self.debug_flag{println!("c.andi");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize] & imm as u64;
        false
    }

    fn c_sub(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("c.sub");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize].wrapping_sub(self.x_reg[rs2 as usize]);
        false
    }

    fn c_xor(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("c.xor");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize] ^ self.x_reg[rs2 as usize];
        false
    }
    fn c_or(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("c.or");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize] ^ self.x_reg[rs2 as usize];
        false
    }
    fn c_and(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("c.and");}
        self.x_reg[rd as usize] = self.x_reg[rd as usize] & self.x_reg[rs2 as usize];
        false
    }

    fn c_subw(&mut self, rd: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("c.subw");}
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
        if self.debug_flag{println!("MULU");}
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
        if self.debug_flag{println!("SLLI");}
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
        if self.debug_flag{println!("SRLTI");}
        if (self.x_reg[rs1 as usize] as i64) < (imm as i64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltiu(&mut self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{println!("SLTIU");}
        if (self.x_reg[rs1 as usize] as u64) < imm {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn sltu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("SLTU");}
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.x_reg[rd as usize] = 1;
        } else {
            self.x_reg[rd as usize] = 0;
        }
        false
    }

    fn mulhsu(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("MULHSU");}
        let left = self.x_reg[rs1 as usize] as i64 as u128;
        let right = self.x_reg[rs2 as usize] as u64 as u128;
        let result = (left.wrapping_mul(right) >> 64) as u64;
        self.x_reg[rd as usize] = result;
        false
    }

    fn srl(&mut self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("{:#08X} srl x{rd}, x{rs1}, x{rs2}",self.pc);}
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
        if self.debug_flag{
            println!("{:#08X} -> sd x{rs2},{imm}(x{rs1})",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let value = self.x_reg[rs2 as usize];
        //self.mmu.write(_memory_address, value, DOUBLE_WORD);
        self.mmu.write_double_word(_memory_address, value);
        false
    }

    fn store_word(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} sw x{rs2}{imm}(x{rs1})",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let value = self.x_reg[rs2 as usize];
        //self.mmu.write(_memory_address, value, WORD);
        self.mmu.write_word(_memory_address, value);
        false
    }

    fn store_half(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} sh x{rs2}{imm}(x{rs1})",self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let value = self.x_reg[rs2 as usize];
        //self.mmu.write(_memory_address, value, HALF);
        self.mmu.write_half(_memory_address, value);
        false
    }

    fn store_byte(self: &mut Self, rs2: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} sb x{rs2},{} (x{rs1})", imm as i64, self.pc);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm as u64);
        let value = self.x_reg[rs2 as usize];
        //self.mmu.write(_memory_address, value, BYTE);
        self.mmu.write_byte(_memory_address, value);
        false
    }
    fn load_word_unsigned(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} lwu x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        //let result = u32::from_le_bytes(self.mmu.read(_memory_address,WORD).try_into().unwrap());
        let result = self.mmu.read_word(_memory_address);
        self.x_reg[rd as usize] = result;
        false
    }
    fn load_double_word(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} ld x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        //let result = u64::from_le_bytes(self.mmu.read(_memory_address,DOUBLE_WORD).try_into().unwrap());
        let result = self.mmu.read_double_word(_memory_address);
        self.x_reg[rd as usize] = result;
        false
    }
    fn load_double_word_atomic(self: &mut Self, rd: u64, rs1: u64) -> bool {
        println!("LR.d");
        false
    }
    fn store_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SC.D");
        false
    }
    fn load_word_atomic(self: &mut Self, rd: u64, rs1: u64) -> bool {
        println!("LR.W");
        false
    }
    fn store_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("SC.W");
        false
    }

    fn maxu_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAXU.W");
        false
    }
    fn minu_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMINU.W");
        false
    }

    fn min_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMIN.W");
        false
    }

    fn max_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAX.W");
        false
    }
    fn xor_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOXOR.W");
        false
    }
    fn or_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOOR.W");
        false
    }
    fn and_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOAND.W");
        false
    }
    fn add_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOADD.W");
        false
    }

    fn minu_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMINU.D");
        false
    }

    fn maxu_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAXU.D");
        false
    }

    fn max_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMAX.D");
        false
    }

    fn min_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOMIN.D");
        false
    }

    fn xor_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOXOR.D");
        false
    }
    fn and_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOAND.D");
        false
    }
    fn or_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOOR.D");
        false
    }
    fn add_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOADD.D");
        false
    }

    fn swap_double_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOSWAP.D");
        false
    }

    fn swap_word_atomic(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        println!("AMOSWAP.W");
        false
    }

    fn load_word(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} lw x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let result = self.mmu.read_word(_memory_address) as i32 as i64 as u64;
        self.x_reg[rd as usize] = result;
        false
    }
    // load 16 bit value
    fn load_half(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} lh x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let result = self.mmu.read_half(_memory_address) as i16 as i64 as u64;
        self.x_reg[rd as usize] = result;
        false
    }
    // load 8 bit value
    fn load_byte(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} lb x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let result = self.mmu.read_byte(_memory_address) as i8 as i64 as u64;
        self.x_reg[rd as usize] = result;
        false
    }
    // load 16 bit value
    fn load_half_u(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} luu x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let result = self.mmu.read_half(_memory_address);
        self.x_reg[rd as usize] = result;
        false
    }
    // load 8 bit value
    fn load_byte_u(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{
            println!("{:#08X} lbu x{rd},{}(x{rs1})",self.pc,imm as i64);
        }
        let _memory_address = self.x_reg[rs1 as usize].wrapping_add(imm);
        let result = self.mmu.read_byte(_memory_address);
        self.x_reg[rd as usize] = result;
        false
    }
    // add immediate
    fn addi(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} addi x{rd}, x{rs1}, {}", self.pc, imm as i64);
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
        if self.debug_flag{println!("and");}
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & self.x_reg[rs2 as usize];
        false
    }
    fn or(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("OR");}
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | self.x_reg[rs2 as usize];
        false
    }
    fn xor(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("XOR");}
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ self.x_reg[rs2 as usize];
        false
    }

    fn bgeu(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        if self.debug_flag{println!("BGEU");}
        if (self.x_reg[rs1 as usize] as u64) >= (self.x_reg[rs2 as usize] as u64) {
            self.pc = self.pc.wrapping_add(imm_b_type as i64 as u64);
            return true;
        }
        false
    }
    fn bltu(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        if self.debug_flag{println!("bltu");}
        if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[rs2 as usize] as u64) {
            self.pc = self.pc.wrapping_add(imm_b_type as i64 as u64);
            return true;
        }
        false
    }

    fn bge(self: &mut Self, rs1: u64, rs2: u64, imm_b_type: u64) -> bool {
        if self.debug_flag{println!("bge");}
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
        if self.debug_flag{println!("bne");}
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
        if self.debug_flag{println!("jal {:#08X}",self.pc.wrapping_add(0x4));}
        self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
        let t = self.pc.wrapping_add(imm_j_type as i64 as u64);
        // check if jumping to return address
        if self.x_reg[1] == t {
            self.call_stack.pop();
        } else {
            let i = self.call_stack.len() - 1;
            self.call_stack[i] = self.pc.wrapping_add(0x4);
            self.call_stack.push(self.pc.wrapping_add(0x4));
        }
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
            println!("c.bnez x{rs1}, x0, {:#08X}", offset);
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
        if self.debug_flag{println!("c.JALR");}
        let t = self.pc + 2;
        if self.x_reg[1] == t {
            self.call_stack.pop();
        } else {
            let i = self.call_stack.len() - 1;
            self.call_stack[i] = self.pc.wrapping_add(t);
            self.call_stack.push(self.pc.wrapping_add(t));
        }
        self.pc = self.x_reg[rs1 as usize];
        self.x_reg[1] = t; // set x1 to ret
        true
    }

    fn jalr(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        let t = self.pc.wrapping_add(4);
        let target = ((self.x_reg[rs1 as usize] as i64).wrapping_add(imm as i64)) & !1;
        // check if target is return address
        if self.x_reg[1] == t {
            self.call_stack.pop();
        } else {
            let i = self.call_stack.len() - 1;
            self.call_stack[i] = self.pc.wrapping_add(t);
            self.call_stack.push(self.pc.wrapping_add(t));
        }
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
            println!("{:#08X} auipc x{} {:#08X}", self.pc, rd, imm_u_type);
        }
        self.x_reg[rd as usize] = self.pc.wrapping_add(imm_u_type);
        false
    }

    fn addiw(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag{println!("ADDIW confirm works");}
        let rs1__ = self.x_reg[rs1 as usize];
        self.x_reg[rd as usize] = rs1__.wrapping_add(imm) as i64 as u64;
        false
    }

    fn slliw(self: &mut Self, rd: u64, rs1: u64, imm: u64) -> bool {
        if self.debug_flag {
            println!("{:#08X} slliw",self.pc);
        }
        let left = self.x_reg[rs1 as usize] as i32;
        let shamt = (imm & 0x1f) as u32;
        self.x_reg[rd as usize] = (left >> shamt) as i64 as u64;
        false
    }

    fn srliw(self: &mut Self, rd: u64, rs1: u64, shamt: u64) -> bool {
        if self.debug_flag{println!("SRLIW");}
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
        if self.debug_flag{println!("DIVUW");}
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
        if self.debug_flag{println!("MULW");}
        let left = self.x_reg[rs1 as usize] as u32;
        let right = self.x_reg[rs2 as usize] as u32;
        let result = left.wrapping_mul(right);
        self.x_reg[rd as usize] = result as i32 as u64;
        false
    }
    fn subw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("SUBW");}
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_sub(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }
    fn addw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("ADDW");}
        self.x_reg[rd as usize] = (self.x_reg[rs1 as usize] as u32)
            .wrapping_add(self.x_reg[rs2 as usize] as u32) as i64
            as u64;
        false
    }

    fn divw(self: &mut Self, rd: u64, rs1: u64, rs2: u64) -> bool {
        if self.debug_flag{println!("DIVW");}
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
        if self.debug_flag{println!("REMW");}
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
            todo!("DEBUG EBREAK");
        }
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
        if self.debug_flag{println!("syscall {:#08X}", syscall);}
        match syscall {
            0xDE => {
                let addr = _a0;
                let length = _a1;
                let prot = _a2;
                let flags = _a3;
                let fd = _a4;
                let offset = _a5;
                println!("MMAP CALLED {:#08X} {:#08X} {:#08X} {:#08X} {:#08X} {:#08X}",addr,length,prot,flags,fd,offset);
                println!("{}",length as u64);
                todo!("MMAP SYSCALL");
                self.mmu.alloc(0x440000, 0x0124,false,false,false);
                self.x_reg[10] = 0x440000;
            }
            0x5E => {
                // https://man7.org/linux/man-pages/man2/exit_group.2.html
                let exit_status = _a0;
                self.exit_called = true;
                self.exit_status = exit_status as i32;
                return false;
            }
            0x42 => {
                //https://man7.org/linux/man-pages/man2/writev.2.html
                let fd = _a0;
                if fd != 1 {
                    todo!("HANDLE FILE DESCRIPTORS");
                }
                let iovec_ptr_start = _a1;
                let iovec_count= _a2;
                let iovec_struct_size = std::mem::size_of::<Iovec>() as u64;
                let total_size = iovec_count * iovec_struct_size;
                let iovec_buffer = self.mmu.get_segment_bytes(iovec_ptr_start,total_size).unwrap();
                let mut writev_n = 0;
                unsafe {
                    for i in 0..iovec_count {
                        let offset = i as isize * 16;
                        let iovec_p: *const Iovec = iovec_buffer.as_ptr().offset(offset) as *const Iovec;
                        let iovec:&Iovec = &*iovec_p;
                        if iovec.iov_base == 0 || iovec.iov_len== 0 {
                            break;
                        }
                        let data_buffer = self.mmu.get_segment_bytes(iovec.iov_base,iovec.iov_len).unwrap();
                        let utf_bytes = core::str::from_utf8_unchecked(data_buffer);
                        //print!("{}", utf_bytes);
                        writev_n += data_buffer.len() as u64;
                    }
                }
                self.x_reg[10] = writev_n;
            }
            0x1D => {
                //https://man7.org/linux/man-pages/man2/ioctl.2.html
                self.x_reg[10] = 0;
            }
            0x62 => {
                // https://man7.org/linux/man-pages/man2/lseek.2.html
                let fd = _a0;
                let offset = _a1;
                let whence = _a2;
                println!("{:#08X} {:#08X} {:#08X}", fd, offset, whence);
                self.x_reg[10] = fd;
                todo!("LSEEK SYSCALL");
            }
            0x60 => {
                // https://man7.org/linux/man-pages/man2/set_tid_address.2.html
                // return value set current pid as value, we dont support threading.
                self.x_reg[10] = std::process::id() as u64;
            }
            0x40 => {
                // write syscall
                let fd = _a0;
                let raw_bytes = self.mmu.get_segment_bytes(_a1,_a2).unwrap();
                unsafe {
                    let utf_bytes = core::str::from_utf8_unchecked(raw_bytes);
                    if fd == 1 || fd == 2 {
                        //print!("{}", utf_bytes);
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
            0x38 => {
                let dirfd = _a0;
                let pathname = _a1;
                let open_how = _a2;
                let size = _a3;
                println!("openat syscall {:#08X} {:#08X} {open_how} {size}",dirfd,pathname);
                // create a a file descriptor and return it.
                //let pathname_string = self.mmu.get_segment_bytes(pathname, size);
                let pathname_string = self.mmu.read_string(dirfd);
                let file = std::fs::File::open(pathname_string).unwrap().as_raw_fd();
                // add to file descriptors
                self.file_descriptors.insert(5,file);
                self.x_reg[10] = file as u64;
            }
            _ => {
                panic!("Unimplemented syscall -> {:#08X}", syscall);
            }
        }
        false
    }
}
