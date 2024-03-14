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
// regs w means always produce 32bit value
// *.w instructions
//
//

// RV64I: base integer instructions
//
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
    fn print_state(self: &Self) {
        // Print the current cpu state registers etc.
    }

    pub fn decode(self: &mut Self, instruction: u32) {
        let opcode = instruction & 0x0000007f;
        let rd = (instruction & 0x00000f80) >> 7;
        let rs1 = (instruction & 0x000f8000) >> 15;
        let rs2 = (instruction & 0x01f00000) >> 20;
        let _funct3 = (instruction & 0x00007000) >> 12;
        let funct7 = (instruction & 0xfe000000) >> 25;
        let imm = ((instruction as i32 as i64) >> 20) as u64;
        let _funct6 = funct7 >> 1;
        //let rs2 =  ((instruction >> 12))
        println!("opcode -> {:#08x}", opcode);
        println!("rd (dest)-> {:#08x}", rd);
        println!("rs1 (src1) -> {:#08x}", rs1);
        println!("rs2 (src2) -> {:#08x}", rs2);
        println!("immediate (immediate) -> {:#08x}", imm);
        self.x_reg[rd as usize] = imm.wrapping_add(rs1 as u64);
        println!("-> {:?}", self.x_reg);
        // opcode needs to be then matched with a funct3 after
        // https://inst.eecs.berkeley.edu/~cs61c/fa18/img/riscvcard.pdf
        // u64 has wrapping add
    }
    // execute instructioon
    pub fn execute(self: &mut Self, instruction: u32) -> bool {
        // I TYPE
        let opcode = instruction & 0x0000007f;
        let rd = (instruction & 0x00000f80) >> 7;
        let rs1 = (instruction & 0x000f8000) >> 15;
        let _rs2 = (instruction & 0x01f00000) >> 20;
        let funct3 = (instruction & 0x00007000) >> 12;
        let funct7 = (instruction & 0xfe000000) >> 25;
        let imm = ((instruction as i32 as i64) >> 20) as u64;
        let imm_5_11_mode = (imm >> 6) & 0b111111;
        let imm_5_11 = imm & 0b111111;
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
            // ADD SUB SHIFT ETC
            0b0110011 => match funct3 {
                0x0 => match funct7 {
                    0x0 => {
                        self.add(rd, rs1, _rs2);
                        false
                    }
                    0x20 => {
                        println!("sub");
                        self.x_reg[rd as usize] =
                            self.x_reg[rs1 as usize].wrapping_sub(self.x_reg[_rs2 as usize]);
                        false
                    }
                    _ => {
                        todo!("Invalid funct7");
                    }
                },
                0x5 => match funct7 {
                    0x0 => {
                        self.x_reg[rd as usize] =
                            self.x_reg[rs1 as usize] >> (self.x_reg[_rs2 as usize] & 0b11111);
                        false
                    }
                    0x20 => {
                        self.x_reg[rd as usize] = self.x_reg[rs1 as usize]
                            >> ((self.x_reg[_rs2 as usize] & 0b11111) as i64) as u64;
                        false
                    }
                    _ => {
                        todo!("Invalid funct7");
                    }
                },
                0x4 => {
                    self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ self.x_reg[_rs2 as usize];
                    false
                }
                0x6 => {
                    self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | self.x_reg[_rs2 as usize];
                    false
                }
                0x7 => {
                    self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & self.x_reg[_rs2 as usize];
                    false
                }
                0x1 => {
                    self.x_reg[rd as usize] =
                        self.x_reg[rs1 as usize] << (self.x_reg[_rs2 as usize] & 0b11111);
                    false
                }
                0x2 => {
                    if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[_rs2 as usize] as i64) {
                        self.x_reg[rd as usize] = 1;
                    } else {
                        self.x_reg[rd as usize] = 0;
                    }
                    false
                }
                0x3 => {
                    if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[_rs2 as usize] as u64) {
                        self.x_reg[rd as usize] = 1;
                    } else {
                        self.x_reg[rd as usize] = 0;
                    }
                    false
                }
                _ => {
                    todo!("INVALID FUNCT3")
                }
            },
            // Load Instructions
            0b0000011 => match funct3 {
                // I TYPE
                0x0 => {
                    self.load_byte(rd, rs1, imm);
                    false
                }
                0x1 => {
                    self.load_half(rd, rs1, imm);
                    false
                }
                0x2 => {
                    self.load_word(rd, rs1, imm);
                    false
                }
                0x4 => {
                    self.load_byte_u(rd, rs1, imm);
                    false
                }
                0x5 => {
                    self.load_half_u(rd, rs1, imm);
                    false
                }
                _ => {
                    todo!("Invalid funct3");
                }
            },
            // Store Instructions
            0b0100011 => match funct3 {
                // Stores are S TYPE everything is the same except the immediate register
                0x0 => {
                    self.store_byte(_rs2, rs1, imm_s_type);
                    false
                }
                0x1 => {
                    self.store_half(_rs2, rs1, imm_s_type);
                    false
                }
                0x2 => {
                    self.store_word(_rs2, rs1, imm_s_type);
                    false
                }
                _ => {
                    todo!("Invalid funct3");
                }
            },
            0b0010011 => match funct3 {
                // I TYPE
                0x0 => {
                    self.addi(rd, rs1, imm);
                    false
                }
                0x4 => {
                    self.xori(rd, rs1, imm);
                    false
                }
                0x6 => {
                    self.ori(rd, rs1, imm);
                    false
                }
                0x7 => {
                    self.andi(rd, rs1, imm);
                    false
                }
                0x1 => {
                    self.x_reg[rd as usize] = self.x_reg[rs1 as usize] << imm_5_11;
                    false
                }
                0x5 => match imm_5_11_mode {
                    0x0 => {
                        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] >> imm_5_11;
                        false
                    }
                    0x20 => {
                        self.x_reg[rd as usize] =
                            ((self.x_reg[rs1 as usize] as i64) >> imm_5_11) as u64;
                        false
                    }
                    _ => {
                        todo!("INVALID IMMEDIATE 5-11");
                    }
                },
                0x2 => {
                    if (self.x_reg[rs1 as usize] as i64) < (imm as i64) {
                        self.x_reg[rd as usize] = 1;
                    } else {
                        self.x_reg[rd as usize] = 0;
                    }
                    false
                }
                0x3 => {
                    if (self.x_reg[rs1 as usize] as i64) < (imm as i64) {
                        self.x_reg[rd as usize] = 1;
                    } else {
                        self.x_reg[rd as usize] = 0;
                    }
                    false
                }
                _ => {
                    todo!("Unimplemented funct3");
                }
            },
            0b1100011 => match funct3 {
                0x0 => {
                    println!("beq");
                    println!(
                        "comparing rs1 {:?} and rs2 {:?} potential PC ",
                        self.x_reg[rs1 as usize], self.x_reg[_rs2 as usize]
                    );
                    if self.x_reg[rs1 as usize] == self.x_reg[_rs2 as usize] {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                0x1 => {
                    println!("bne");
                    if self.x_reg[rs1 as usize] != self.x_reg[_rs2 as usize] {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                0x4 => {
                    println!("blt");
                    if (self.x_reg[rs1 as usize] as i64) < (self.x_reg[_rs2 as usize] as i64) {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                0x5 => {
                    println!("bge");
                    if (self.x_reg[rs1 as usize] as i64) >= (self.x_reg[_rs2 as usize] as i64) {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                0x6 => {
                    println!("bltu");
                    if (self.x_reg[rs1 as usize] as u64) < (self.x_reg[_rs2 as usize] as u64) {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                0x7 => {
                    println!("bgeu");
                    if (self.x_reg[rs1 as usize] as u64) >= (self.x_reg[_rs2 as usize] as u64) {
                        self.pc += imm_b_type as i64 as u64;
                        return true;
                    }
                    false
                }
                _ => {
                    todo!("PANIC INVAID OPCODE");
                }
            },
            0b1101111 => {
                // J TYPE
                println!("JAL PC: {:#08X}", self.pc);
                self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
                println!("JAL PC: {:#08X}", self.pc);
                println!("{:#08X} + {:#08X}", imm_j_type, self.pc);
                self.pc = self.pc.wrapping_add(imm_j_type as i64 as u64);
                println!("RET: {:#08X} PC: {:#08X}", self.x_reg[rd as usize], self.pc);
                true
            }
            0b1100111 => {
                // I TYPE
                println!("JALR");
                self.x_reg[rd as usize] = self.pc.wrapping_add(0x4); // return address saved in RD
                self.pc = imm.wrapping_add(self.x_reg[rs1 as usize]); // PC = RS1 + IMM
                true
            }
            0b0110111 => {
                // U TYPE
                self.x_reg[rd as usize] = imm_u_type as i64 as u64;
                false
            }
            0b0010111 => {
                // UTYPE
                self.x_reg[rd as usize] = (imm_u_type as i64 as u64).wrapping_add(self.pc);
                false
            }
            0b1110011 => match funct7 {
                0x0 => {
                    self.ecall();
                    false
                }
                0x1 => {
                    todo!("EBREAK");
                }
                _ => {
                    panic!("INVALID FUNC7 for ECALL OR EBREAK");
                }
            },
            _ => {
                todo!("Unimplemented OpCode");
            }
        }
        // match on opcode then match on func3?
    }
    fn add(self: &mut Self, rd: u32, rs1: u32, rs2: u32) {
        if self.debug_flag {
            println!(
                "ADD x{rd} ({:#08X}) x{rs1} ({:#08X}) x{rs2} ({:#08X})",
                self.x_reg[rd as usize], self.x_reg[rs1 as usize], self.x_reg[rs2 as usize]
            );
        }
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize].wrapping_add(self.x_reg[rs2 as usize]);
    }

    fn store_word(self: &mut Self, rs2: u32, rs1: u32, imm: i32) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u32;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 4]
            .copy_from_slice(&value_as_bytes);
    }
    fn store_half(self: &mut Self, rs2: u32, rs1: u32, imm: i32) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u16;
        let value_as_bytes = value.to_le_bytes();
        self.mmu.memory_segment[_memory_address as usize.._memory_address as usize + 2]
            .copy_from_slice(&value_as_bytes);
    }
    fn store_byte(self: &mut Self, rs2: u32, rs1: u32, imm: i32) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        let value = self.x_reg[index] as u8;
        if self.debug_flag {
            println!("SB {:#08X} <- {:#08X}", _memory_address, value)
        }
        self.mmu.memory_segment[_memory_address as usize] = value;
    }
    fn load_word(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let value3 = self.mmu.memory_segment[_memory_address as usize + 2] as u8;
        let value4 = self.mmu.memory_segment[_memory_address as usize + 3] as u8;
        let result = u32::from_le_bytes([value1, value2, value3, value4]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
    }
    // load 16 bit value
    fn load_half(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]) as i64 as u64;
        self.x_reg[rd as usize] = result as u64;
    }
    // load 8 bit value
    fn load_byte(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value = self.mmu.memory_segment[_memory_address as usize] as u8;
        if self.debug_flag {
            println!(
                "LB x{rd} ({:#08X}) {:#08X} -> ({:#08X})",
                self.x_reg[rd as usize], _memory_address, value
            );
        }

        self.x_reg[rd as usize] = value as u64;
    }
    // load 16 bit value
    fn load_half_u(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value1 = self.mmu.memory_segment[_memory_address as usize] as u8;
        let value2 = self.mmu.memory_segment[_memory_address as usize + 1] as u8;
        let result = u16::from_le_bytes([value1, value2]) as u64;
        self.x_reg[rd as usize] = result as u64;
    }
    // load 8 bit value
    fn load_byte_u(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let value = self.mmu.memory_segment[_memory_address as usize] as u8;
        self.x_reg[rd as usize] = value as u64;
    }
    // add immediate
    fn addi(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        let rs1_value = self.x_reg[rs1 as usize];
        if self.debug_flag {
            println!(
                "ADDI x{rd} ({:#08X}) x{rs1} ({:#08X}) imm ({:#08X})",
                self.x_reg[rd as usize], self.x_reg[rs1 as usize], imm
            );
        }
        self.x_reg[rd as usize] = imm.wrapping_add(self.x_reg[rs1 as usize]);
    }
    fn andi(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] & imm;
    }
    fn ori(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] | imm;
    }
    fn xori(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        self.x_reg[rd as usize] = self.x_reg[rs1 as usize] ^ imm;
    }
    fn ecall(self: &mut Self) {
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
                        todo!("Handle other file descriptors");
                    }
                }
            }
            0x5D => {
                println!("\n=== CoffeePot Exit! ===");
                std::process::exit(_a0 as i32);
            }
            _ => {
                todo!("Unimplemented syscall");
            }
        }
    }
}
