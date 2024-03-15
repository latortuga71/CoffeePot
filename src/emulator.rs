use crate::{cpu::CPU, loader::ElfInformation, mmu::Segment};

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

    // copies instruction from memory into our current instruction u32
    pub fn fetch_instruction(self: &mut Self) -> bool {
        let start = self.cpu.pc;
        let end = self.cpu.pc + 0x4;
        let text_section_size = self.cpu.mmu.text_segment.len();
        if end > text_section_size as u64 {
            panic!("END OF TEXT SECTION EXITING");
        }
        let instruction_bytes = &self.cpu.mmu.text_segment[start as usize..end as usize];
        let mut sliced: [u8; 4] = [0, 0, 0, 0];
        sliced.copy_from_slice(instruction_bytes);
        self.current_instruction = Emulator::as_u32_le(&sliced);
        // return false if the next instruction points to nothing
        self.current_instruction != 0
    }

    pub fn execute_instruction(self: &mut Self) {
        if !self.cpu.execute(self.current_instruction) {
            // IF NO BRANCH WAS TAKEN WE INCREMENT PC
            self.cpu.pc += 0x4;
        }
        // always set x0 to the SP
        self.cpu.sp = self.cpu.x_reg[2] as u32;
        // always set x0 to zero
        self.cpu.x_reg[0] = 0x0;
    }

    pub fn load_elf_segments(self: &mut Self, elf: &ElfInformation) {
        let mut c = 0;
        if self.cpu.mmu.virtual_memory.len() < elf.virtual_memory_minimum_size as usize {
            println!("RESIZED");
            self.cpu
                .mmu
                .virtual_memory
                .resize(elf.virtual_memory_minimum_size as usize, 0);
        }
        for e in &elf.segments {
            c += 1;
            // from offset to end
            let offset = e.virtual_address as usize;
            let offset_end = e.raw_data.len() + offset;
            // copy raw_data into virtual memory
            self.cpu.mmu.virtual_memory[offset..offset_end].copy_from_slice(&e.raw_data);
        }
        println!("copied {c} segments");
    }

    pub fn load_raw_instructions(self: &mut Self, path: &str) -> Result<(), std::io::Error> {
        let bytes = match std::fs::read(path) {
            Ok(bytes) => bytes,
            Err(e) => return Err(e),
        };
        // Load bytes Into Flat Memory Space For Now
        self.cpu.mmu.text_segment = bytes;
        // Set PC at start of bytes
        self.cpu.pc = 0x0000000;
        Ok(())
    }
}
