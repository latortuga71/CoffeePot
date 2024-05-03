use crate::{cpu::CPU, loader::ElfInformation};

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

    pub fn snapshot(&self) -> Emulator {
        self.clone()
    }

    pub fn restore (&mut self, original:&Emulator){
        // restore registers
        self.cpu.x_reg = original.cpu.x_reg;
        self.cpu.pc = original.cpu.pc;
        self.cpu.sp = original.cpu.sp;
        self.cpu.exit_status = 0;
        self.cpu.exit_called = false; // reset this everytime
        // restore fd
        //self.cpu.file_descriptors = original.cpu.file_descriptors.clone();
        // restore dirty memory
        // get dirty segments and change the mout
        for address in self.cpu.mmu.get_dirty_segments() {
            self.cpu.mmu.get_segment(address.0).unwrap().data.copy_from_slice(&original.cpu.mmu.virtual_memory.get(&address).unwrap().data);
        }
    }

    pub fn fetch_instruction(self: &mut Self) -> bool {
        // check if can execute
        if !self.cpu.mmu.get_segment(self.cpu.pc).unwrap().executable() {
            todo!("TODO LOG invalid execution")
        }
        let instruction_bytes = self.cpu.mmu.read_word(self.cpu.pc);
        self.current_instruction = instruction_bytes as u32;
        self.current_instruction != 0
    }

    pub fn execute_instruction(self: &mut Self) -> bool {
        if (0x3 & self.current_instruction) != 0x3 {
            if !self.cpu.execute_compressed(self.current_instruction as u64) {
                self.cpu.pc += 0x2;
            }
            self.cpu.was_last_compressed = true;
        } else {
            if !self.cpu.execute(self.current_instruction as u64) {
                self.cpu.pc += 0x4;
            }
            self.cpu.was_last_compressed = false;
        }
        self.cpu.sp = self.cpu.x_reg[2];
        self.cpu.x_reg[0] = 0x0;
        self.cpu.exit_called
    }


    pub fn load_elf_segments(self: &mut Self, elf: &ElfInformation) {
        self.cpu.mmu.alloc(elf.code_segment_start,elf.code_segment_size as usize,true,true,true);
        for e in &elf.segments {
            let s = self.cpu.mmu.get_segment(e.virtual_address).unwrap();
            let offset = e.virtual_address.wrapping_sub(s.base_address) as usize;
            let offset_end = offset.wrapping_add(e.raw_data.len());
            s.data[offset..offset_end].copy_from_slice(&e.raw_data);
            //s.writable = false; // code shouldn't be writable after first load? libc issues?
        }
    }

    pub fn initialize_stack_libc(self: &mut Self, argc:u64, argv0: String) -> u64 { 
        // TODO ACTUALLY USE ARGC AND LOOP OVER ARGV0
        let stack_base: u64 = 0x020000;
        let stack_end: u64 = stack_base.wrapping_add(1024 * 1024);
        let mut sp = stack_end;
        self.cpu.mmu.alloc(stack_base, 1024*1024,true,true,false);
        let allocation_address = self.cpu.mmu.alloc(0, 0x1024,true,true,false); 
        self.cpu.mmu.write_double_word(allocation_address, 0x4141414141414141);
        self.cpu.mmu.write_double_word(sp,1u64);
        sp -= 8;
        self.cpu.mmu.write_double_word(sp,0x99);
        sp -= 8;
        // zeros
        self.cpu.mmu.write_double_word(sp,0u64);
        sp -= 8;
        self.cpu.mmu.write_double_word(sp,0u64);
        sp -= 8;
        self.cpu.mmu.write_double_word(sp,0u64);
        sp
    }
}
