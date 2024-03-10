use crate::mmu::MMU;

#[derive(Debug)]
pub struct CPU {
    pub pc: u64,
    pub sp: u32,
    pub mmu: MMU,
    pub x_reg: [u64; 32],
}

// XLEN = u64 arch size
// regs w means always produce 32bit value
// *.w instructions

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
    pub fn execute(self: &mut Self, instruction: u32) {
        // I TYPE
        let opcode = instruction & 0x0000007f;
        let rd = (instruction & 0x00000f80) >> 7;
        let rs1 = (instruction & 0x000f8000) >> 15;
        let _rs2 = (instruction & 0x01f00000) >> 20;
        let funct3 = (instruction & 0x00007000) >> 12;
        let funct7 = (instruction & 0xfe000000) >> 25;
        let imm = ((instruction as i32 as i64) >> 20) as u64;
        let _funct6 = funct7 >> 1;

        // S TYPE IMMEDIATE VALUE
        let imm115 = (instruction >> 25) & 0b1111111;
        let imm40 = (instruction >> 7) & 0b11111;
        let imm_s = (imm115 << 5) | imm40;
        let imm_s_type = ((imm_s as i32) << 20) >> 20;
        match opcode {
            0b00000000 => {
                todo!("Invalid Memory Error!");
            }
            // Load Instructions
            0b0000011 => match funct3 {
                0x0 => {
                    todo!("LB");
                }
                0x1 => {
                    todo!("LH");
                }
                0x2 => {
                    todo!("LW");
                }
                0x4 => {
                    todo!("LBU");
                }
                0x5 => {
                    todo!("LHU");
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
                }
                0x1 => {
                    todo!("SH");
                }
                0x2 => {
                    todo!("SW");
                }
                _ => {
                    todo!("Invalid funct3");
                }
            },
            0b011011 => match funct3 {
                0x0 => {
                    println!("ADD");
                }
                _ => {
                    todo!("Unimplemented funct3")
                }
            },
            0b0010011 => match funct3 {
                0x0 => {
                    self.addi(rd, rs1, imm);
                }
                0x4 => {
                    println!("XORI");
                    self.xori(rd, rs1, imm);
                }
                0x6 => {
                    println!("ORI");
                    self.ori(rd, rs1, imm);
                }
                0x7 => {
                    println!("ANDI");
                    self.andi(rd, rs1, imm);
                }
                _ => {
                    todo!("Unimplemented funct3");
                }
            },
            0b1110011 => {
                self.ecall();
            }
            _ => {
                todo!("Unimplemented OpCode");
            }
        }
        // match on opcode then match on func3?
    }
    fn store_byte(self: &mut Self, rs2: u32, rs1: u32, imm: i32) {
        let _memory_address = self.x_reg[rs1 as usize] + imm as u64;
        let index = rs2 as usize;
        //println!("storing at this memory location {:?}", _memory_address);
        self.mmu.memory_segment[_memory_address as usize] = self.x_reg[index] as u8;
    }
    // add immediate
    fn addi(self: &mut Self, rd: u32, rs1: u32, imm: u64) {
        //println!("rd: {:?} rs1: {:?} imm: {:?}", rd, rs1, imm);
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
                let end = _a1 + _a2;
                let raw_bytes = &self.mmu.memory_segment[_a1 as usize..end as usize];
                let utf_bytes = core::str::from_utf8(raw_bytes).unwrap();
                todo!("Handle other file descriptors");
                // print bytes
                print!("{}", utf_bytes);
                // set reuturn value
                self.x_reg[10] = _a2;
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
