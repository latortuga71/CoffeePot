#[derive(Debug)]
pub struct MMU {
    // Make this one giant array
    // and make sure each segement is placed accordingly in the array to simulate virtual memory
    pub text_segment: Vec<u8>,
    pub memory_segment: Vec<u8>,
    pub virtual_memory: Vec<u8>,
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            text_segment: vec![0; 1024], // where our
            memory_segment: vec![0; 0xFFFF],
            virtual_memory: vec![0; 0xFFFFFFFF], // 4MB of address space by default
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
