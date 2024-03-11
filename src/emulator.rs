use crate::cpu::CPU;

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
        self.cpu.execute(self.current_instruction);
        self.cpu.pc += 0x4;
        self.cpu.sp = self.cpu.x_reg[2] as u32;
    }

    pub fn load_elf(path: &str) {
        todo!("Load Elf");
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
