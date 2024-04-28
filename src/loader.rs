use crate::mmu::{ElfSection};
use std::{io::Write, process::{self, exit}};

#[derive(Debug)]
pub struct ElfInformation {
    pub segments: Vec<ElfSection>,
    pub entry_point: u64,
    pub virtual_memory_minimum_size: u64,
    // TODO OTHER STUFF THAT WE NEED?
}

impl ElfInformation {
    pub fn load_segments(self: &Self) -> bool {
        todo!("For Each Section Load Sections Into Memory");
        true
    }
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
    let mut segments: Vec<ElfSection> = Vec::new();
    let mut virtual_memory_minimum_size = 0;
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
        let segment_end = segment_start as usize + program_header.p_memsz as usize;
        virtual_memory_minimum_size += program_header.p_memsz;
        virtual_memory_minimum_size += program_header.p_vaddr;
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
            if debug {
                println!("Segment type {:#08X}", program_header.p_type);
                println!("Segment align {:#08X}", program_header.p_align);
                println!("Segment virtual addr {:#08X}", program_header.p_vaddr);
                println!("Segment vm size {:#08X}", program_header.p_memsz);
                println!("Segment raw size {:#08X}", program_header.p_filesz);
            }
        }
    }
    ElfInformation {
        segments,
        entry_point,
        virtual_memory_minimum_size,
    }
}
