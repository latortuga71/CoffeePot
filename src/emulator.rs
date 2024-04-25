use std::{clone, io::Write};

use crate::{cpu::CPU, loader::ElfInformation, mmu::Segment};

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
        let start = self.cpu.pc;
        let end = self.cpu.pc + 0x4;
        let instruction_bytes = self.cpu.mmu.read(self.cpu.pc,0x4);
        //let instruction_bytes = &self.cpu.mmu.virtual_memory[start as usize..end as usize];
        let mut sliced: [u8; 4] = [0, 0, 0, 0];
        sliced.copy_from_slice(instruction_bytes);
        self.current_instruction = Emulator::as_u32_le(&sliced);
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
            self.cpu.mmu.virtual_memory[offset..offset_end].copy_from_slice(&e.raw_data);
        }
        //println!("copied {c} segments");

    }
    pub fn load_elf_segments_into_mmu(self: &mut Self, elf: &ElfInformation) {
        for e in &elf.segments {
            let offset = e.virtual_address;
            let offset_end = e.raw_data.len() as u64 + offset;
            let k = (offset,offset_end);
            self.cpu.mmu.alloc(offset, e.raw_data.len().try_into().unwrap());
            //e.raw_data.copy_from_slice(&self.cpu.mmu.virtual_memory_new.get_mut(&k).unwrap().data[offset as usize..offset_end as usize]);
            self.cpu.mmu.virtual_memory_new.get_mut(&k).unwrap().data.copy_from_slice(&e.raw_data);
            //println!("{wrote} base address {:#08X}",offset);
        }
    }

    pub fn load_raw_instructions(self: &mut Self, path: &str) -> Result<(), std::io::Error> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(e),
        };
        // Load bytes Into Flat Memory Space For Now
        //self.cpu.mmu.text_segment = bytes;
        self.cpu.mmu.virtual_memory.resize(bytes.len(), 0);
        self.cpu.mmu.virtual_memory.copy_from_slice(&bytes);
        // Set PC at start of bytes
        self.cpu.pc = 0x0000000;
        Ok(())
    }
}
