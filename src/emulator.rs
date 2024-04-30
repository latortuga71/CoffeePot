use std::{clone, io::Write};

use crate::{cpu::CPU, data, loader::ElfInformation, mmu::{self, Segment}};

#[derive(Clone)]
pub struct Emulator {
    pub cpu: CPU,
    pub current_instruction: u32,
}

impl Emulator {
    pub fn new() -> Self {
        Emulator {
            cpu: CPU::new(),
            current_instruction: 0,
        }
    }

    fn as_u32_be(array: &[u8; 4]) -> u32 {
        ((array[0] as u32) << 24)
            + ((array[1] as u32) << 16)
            + ((array[2] as u32) << 8)
            + ((array[3] as u32) << 0)
    }

    fn as_u32_le(array: &[u8; 4]) -> u32 {
        ((array[0] as u32) << 0)
            + ((array[1] as u32) << 8)
            + ((array[2] as u32) << 16)
            + ((array[3] as u32) << 24)
    }

    // copies u32 bit instruction from memory into our current instruction
    pub fn fetch_instruction(self: &mut Self) -> bool {
        // fetch small instructino
        let start = self.cpu.pc;
        let end = self.cpu.pc + 0x4;
        //let instruction_bytes = self.cpu.mmu.read_to_exec(self.cpu.pc,mmu::WORD);
        let instruction_bytes = self.cpu.mmu.read_word(self.cpu.pc);
        //let mut sliced: [u8; 4] = [0, 0, 0, 0];
        //sliced.copy_from_slice(instruction_bytes);
        self.current_instruction = instruction_bytes as u32;
        // return false if the next instruction points to nothing
        self.current_instruction != 0
    }

    pub fn execute_instruction(self: &mut Self) -> bool {
        // Here we can check if its a compressed instruction
        if (0x3 & self.current_instruction) != 0x3 {
            // compressed instruction
            if !self.cpu.execute_compressed(self.current_instruction as u64) {
                self.cpu.pc += 0x2; // no branch increment PC
            }
            self.cpu.was_last_compressed = true;
        } else {
            // not compressed
            if !self.cpu.execute(self.current_instruction as u64) {
                self.cpu.pc += 0x4; // no branch increment PC
            }
            self.cpu.was_last_compressed = false;
        }
        // always set x2 to the SP
        self.cpu.sp = self.cpu.x_reg[2];
        // always set x0 to zero
        self.cpu.x_reg[0] = 0x0;
        self.cpu.exit_called
    }

    pub fn load_elf_segments(self: &mut Self, elf: &ElfInformation) {
        // load elf LOAD sections into 1 memory segment in the mmu
        for e in &elf.segments {
            // from offset to end
            let offset = e.virtual_address as usize;
            let offset_end = e.raw_data.len() + offset;
            // copy raw_data into virtual memory
            self.cpu.mmu.virtual_memory_new[offset..offset_end].copy_from_slice(&e.raw_data);
            //println!("CODE SECTION -> {:#08X} {:#08X}",offset,offset_end)
        }
        //println!("copied {c} segments");
    }

    pub fn load_elf_segments_new(self: &mut Self, elf: &ElfInformation) {
        for e in &elf.segments {
            let offset = e.virtual_address as usize;
            self.cpu.mmu.alloc(e.virtual_address,e.raw_data.len());
            self.cpu.mmu.get_segment(e.virtual_address).unwrap().data.copy_from_slice(&e.raw_data);
        }
    }
    pub fn initialize_stack_libc(self: &mut Self, argc:u64, argv0: String) ->u64 { 
        // TODO ACTUALLY USE ARGC AND LOOP OVER ARGV0
        let stack_base:u64 = 0x020000;
        let stack_end:u64 = stack_base.wrapping_add(1024 * 1024);
        let mut sp = stack_end;
        self.cpu.mmu.alloc(stack_base, 1024*1024);
        let allocation_address = self.cpu.mmu.alloc(0, 0x1024); 
        self.cpu.mmu.write_double_word_new(allocation_address, 0x4141414141414141);
        self.cpu.mmu.write_double_word_new(sp,1u64);
        sp -= 8;
        self.cpu.mmu.write_double_word_new(sp,0x99);
        sp -= 8;
        // zeros
        self.cpu.mmu.write_double_word_new(sp,0u64);
        sp -= 8;
        self.cpu.mmu.write_double_word_new(sp,0u64);
        sp -= 8;
        self.cpu.mmu.write_double_word_new(sp,0u64);
        sp
    }

    pub fn load_raw_instructions(self: &mut Self, path: &str) -> Result<(), std::io::Error> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(e),
        };
        // Load bytes Into Flat Memory Space For Now
        //self.cpu.mmu.text_segment = bytes;
        self.cpu.mmu.virtual_memory_new.resize(bytes.len(), 0);
        self.cpu.mmu.virtual_memory_new.copy_from_slice(&bytes);
        // Set PC at start of bytes
        self.cpu.pc = 0x0000000;
        Ok(())
    }
}
