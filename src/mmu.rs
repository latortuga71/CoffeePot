#[derive(Debug)]
pub struct MMU {
    pub text_segment: Vec<u8>,
    pub memory_segment: Vec<u8>,
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            text_segment: vec![0; 1024],
            memory_segment: vec![0; 0xFFFF],
        }
    }
}
