use crate::mmu::{ElfSection};
use std::{io::Write, process::{self, exit}};

#[derive(Debug)]
pub struct ElfInformation {
    pub code_segment_start: u64,
    pub code_segment_size: u64,
    pub segments: Vec<ElfSection>,
    pub entry_point: u64,
}

// https://ics.uci.edu/~aburtsev/238P/hw/hw3-elf/hw3-elf.html
#[repr(C)]
pub struct Elf64FileHeader {
    pub e_ident: [u8; 16],
    pub e_type: u16,
    pub e_machine: u16,
    pub e_version: u32,
    pub e_entry: u64,
    pub e_phoff: u64,
    pub e_shoff: u64,
    pub e_flags: u32,
    pub e_ehsize: u16,
    pub e_phentsize: u16,
    pub e_phnum: u16,
    pub e_shentsize: u16,
    pub e_shnum: u16,
    pub e_shstrndx: u16,
}

#[repr(C)]
pub struct Elf64ProgramHeader {
    pub p_type: u32,
    pub p_flags: u32,
    pub p_offset: u64,
    pub p_vaddr: u64,
    pub p_paddr: u64,
    pub p_filesz: u64,
    pub p_memsz: u64,
    pub p_align: u64,
}

//https://docs.rs/elf/latest/elf/file/struct.FileHeader.html
pub fn load_elf(path: &str,debug: bool) -> ElfInformation {
    // TODO FIX THE ISSUE WITH OVERALLOCATING THE CODE SECTION SIZE SLIGHTLY
    let mut segments: Vec<ElfSection> = Vec::new();
    let mut virtual_memory_minimum_size = 0;
    let mut previous_segment_end = 0x0;
    let file_data = std::fs::read(path).unwrap_or_else(|e| {
        eprint!("ERROR: Failed to read {path}: {e}");
        exit(1);
    });
    let file_header: Elf64FileHeader = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };
    let entry_point = file_header.e_entry;
    let program_header_offset = file_header.e_phoff;
    if debug{
        println!("{:?}", file_header.e_ident);
        println!("{:#08x}", entry_point);
        println!("{:?}", program_header_offset);
    }
    for i in 0..file_header.e_phnum {
        let offset_start = program_header_offset + (file_header.e_phentsize * i) as u64;
        let offset_end = offset_start + file_header.e_phentsize as u64;
        let program_header: Elf64ProgramHeader = unsafe {
            std::ptr::read(
                file_data[offset_start as usize..offset_end as usize].as_ptr() as *const _,
            )
        };

        let segment_start = (program_header.p_offset) as usize;
        let mut segment_end = segment_start as usize + program_header.p_filesz as usize; // has to be filesz
        let segment_size = segment_end.wrapping_sub(segment_start);
        if previous_segment_end != 0 {
            virtual_memory_minimum_size = (virtual_memory_minimum_size as u64).wrapping_add(program_header.p_vaddr.wrapping_sub(previous_segment_end));
        }
        previous_segment_end = program_header.p_vaddr + program_header.p_memsz;
        virtual_memory_minimum_size += program_header.p_vaddr - program_header.p_memsz;
        if program_header.p_type == 0x1 {
            let segment = ElfSection {
                alignment: program_header.p_align,
                perms: program_header.p_flags as u64,
                raw_data_size: program_header.p_filesz,
                raw_data: Vec::from(&file_data[segment_start..segment_end]),
                virtual_memory_size: program_header.p_memsz,
                virtual_address: program_header.p_vaddr,
            };
            segments.push(segment);
        }
    }
    let code_start = segments.clone().get(0).unwrap().virtual_address;
    ElfInformation {
        segments,
        entry_point,
        code_segment_start: code_start,
        code_segment_size:virtual_memory_minimum_size
    }
}
