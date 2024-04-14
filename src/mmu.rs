#[derive(Debug)]
pub struct MMU {
    // Make this one giant array
    pub virtual_memory: Vec<u8>,
}

pub const RAM: u64 = 1024 * 1024 * 1024;

impl MMU {
    pub fn new() -> Self {
        MMU {
            virtual_memory: vec![0; RAM as usize], // 1GB of address space by default
        }
    }
}

#[derive(Debug)]
pub struct Segment {
    pub raw_data: Vec<u8>,
    pub raw_data_size: u64,
    pub virtual_address: u64,
    pub virtual_memory_size: u64,
    pub perms: u64,
    pub alignment: u64,
}
