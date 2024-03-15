use std::process::exit;

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
pub fn load_elf(path: &str) {
    let file_data = std::fs::read(path).unwrap_or_else(|e| {
        eprint!("ERROR: Failed to read {path}: {e}");
        exit(1);
    });
    let file_header: Elf64FileHeader = unsafe { std::ptr::read(file_data.as_ptr() as *const _) };
    let entry_point = file_header.e_entry;
    let program_header_offset = file_header.e_phoff;
    println!("{:?}", file_header.e_ident);
    println!("{:#08x}", entry_point);
    println!("{:?}", program_header_offset);
    let program_header_size = file_header.e_phentsize * file_header.e_phnum;
    // Program Header
    for i in 0..file_header.e_phnum {
        let offset_start = program_header_offset + (file_header.e_phentsize * i) as u64;
        let offset_end = offset_start + file_header.e_phentsize as u64;
        let program_header: Elf64ProgramHeader = unsafe {
            std::ptr::read(
                file_data[offset_start as usize..offset_end as usize].as_ptr() as *const _,
            )
        };
        println!("start {:#08X} end {:#08X}", offset_start, offset_end);
        println!("TYPE {:#08X}", program_header.p_type);
        println!(
            "type: {:#08X} virt addr:{:#08X} memsize {:#08X} align: {:#08X}\n",
            program_header.p_type,
            program_header.p_vaddr,
            program_header.p_memsz,
            program_header.p_align
        );
    }
}
