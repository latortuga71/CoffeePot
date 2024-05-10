use std::{collections::HashMap, error::Error, f32::consts::E, fmt, fs::read, io::Write, thread::panicking};

// Error for mmu
#[derive(Debug)]
pub struct MMUError{
    error_msg: String,
}

impl MMUError {
    fn new(msg:&str ) -> MMUError {
        MMUError{
            error_msg:msg.to_string()
        }
    }
}
impl fmt::Display for MMUError {
    fn fmt(&self,f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.error_msg)
    }
}

impl Error for MMUError {
    fn description(&self) -> &str {
        &self.error_msg
    }
}

#[derive(Debug,Clone)]
pub struct MMU {
    pub virtual_memory: HashMap<(u64,u64),Segment>,
    pub next_alloc_base: u64
}

impl MMU {
    pub fn new() -> Self {
        MMU {
            virtual_memory: HashMap::new(),
            next_alloc_base:0x0,
        }
    }
    pub fn print_segments(&self) {
        for (k,_segment) in self.virtual_memory.iter() {
            print!("Segment -> {:#08X} {:#08X} ",k.0,k.1);
            if _segment.readable {print!("R")}
            if _segment.writable{print!("W")}
            if _segment.executable{print!("X")}
            println!();
        }
    }
    pub fn get_segments(&mut self) -> Vec<(u64,u64)> {
        let mut dirty_segments:Vec<(u64,u64)> = Vec::new();
        for (k,_segment) in self.virtual_memory.iter_mut() {
                let end = _segment.base_address.wrapping_add(_segment.data_size as u64);
                dirty_segments.push((_segment.base_address,end));
        }
        dirty_segments
    }

    pub fn get_dirty_segments(&mut self) -> Vec<(u64,u64)> {
        let mut dirty_segments:Vec<(u64,u64)> = Vec::new();
        for (k,_segment) in self.virtual_memory.iter_mut() {
            if _segment.dirty {
                let end = _segment.base_address.wrapping_add(_segment.data_size as u64);
                dirty_segments.push((_segment.base_address,end));
            }
        }
        dirty_segments
    }

    pub fn get_segment_immut(&self, address:u64) -> Result<&Segment,MMUError> {
        for (k,_segment) in self.virtual_memory.iter() {
            if address >= k.0 && address < k.1 {
                return Ok(_segment);
            }
        }
        let error = format!("TODO! log these Segmentation fault attempting to access {:#08X}",address);
        return Err(MMUError::new(&error));
    }

    pub fn get_segment(&mut self,address:u64) -> Result<&mut Segment,MMUError> {
        for (k,_segment) in self.virtual_memory.iter_mut() {
            if address >= k.0 && address < k.1 {
                return Ok(_segment);
            }
        }
        let error = format!("TODO! log these Segmentation fault attempting to access {:#08X}",address);
        return Err(MMUError::new(&error));
    }

    pub fn get_segment_bytes(&self,address:u64,length:u64) -> Result<&[u8],MMUError> {
        for (k,_segment) in self.virtual_memory.iter() {
            if address >= k.0 && address < k.1 {
                let start= address.wrapping_sub(_segment.base_address) as usize;
                let end = start.wrapping_add(length as usize) as usize;
                return Ok(&_segment.data[start..end])
            }
        }
        let error = format!("Segmentation fault attempting to access {:#08X}",address);
        return Err(MMUError::new(&error));
    }

    fn segment_taken(&self,address:u64) -> bool {
        for (k,_segment) in self.virtual_memory.iter() {
            if address >= k.0 && address <= k.1 {
                return true;
            }
        }
        return false;
    }



    pub fn write_string(&mut self, address:u64,value:&String){
        let segment = self.get_segment(address).unwrap();
        segment.dirty = true;
        if !segment.writable() {
            todo!("LOG INVALID MEMORY PERM ACCESS {:#08X} {:#08X}", address,segment.base_address)
        }
        let start= address.wrapping_sub(segment.base_address) as usize;
        let end = start + value.as_bytes().len();
        segment.data[start..end].copy_from_slice(value.as_bytes());
    }

    pub fn write_double_word(&mut self, address:u64, value:u64){
        let segment = self.get_segment(address).unwrap();
        segment.dirty = true;
        if !segment.writable() {
            todo!("LOG INVALID MEMORY PERM ACCESS {:#08X} {:#08X}", address,segment.base_address)
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        segment.data[addr] = (value & 0xff) as u8;
        segment.data[addr + 1] = ((value >> 8) & 0xff) as u8;
        segment.data[addr + 2] = ((value >> 16) & 0xff) as u8;
        segment.data[addr + 3] = ((value >> 24) & 0xff) as u8;
        segment.data[addr + 4] = ((value >> 32) & 0xff) as u8;
        segment.data[addr + 5] = ((value >> 40) & 0xff) as u8;
        segment.data[addr + 6] = ((value >> 48) & 0xff) as u8;
        segment.data[addr + 7] = ((value >> 56) & 0xff) as u8;
    }

    pub fn write_word(&mut self, address:u64, value:u64){
        let segment = self.get_segment(address).unwrap();
        segment.dirty = true;
        if !segment.writable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        segment.data[addr] = (value & 0xff) as u8;
        segment.data[addr + 1] = ((value >> 8) & 0xff) as u8;
        segment.data[addr + 2] = ((value >> 16) & 0xff) as u8;
        segment.data[addr + 3] = ((value >> 24) & 0xff) as u8;
    }

    pub fn write_half(&mut self, address:u64, value:u64){
        let segment = self.get_segment(address).unwrap();
        segment.dirty = true;
        if !segment.writable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        segment.data[addr] = (value & 0xff) as u8;
        segment.data[addr + 1] = ((value >> 8) & 0xff) as u8;
    }

    pub fn write_byte(&mut self, address:u64, value:u64){
        let segment = self.get_segment(address).unwrap();
        segment.dirty = true;
        if !segment.writable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        segment.data[addr] = value as u8;
    }

    pub fn read_string(&mut self, address:u64) -> String {
        let segment = self.get_segment(address).unwrap();
        if !segment.readable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        let mut i = 0;
        let mut string_bytes:Vec<u8> = vec![0;0];
        loop {
            if segment.data[addr + i] == 0x0 {
                break;
            }
            string_bytes.push(segment.data[addr + i]);
            i += 1;
        }
        std::str::from_utf8(&string_bytes).unwrap().to_string()
    }

    pub fn read_double_word(&mut self, address:u64) -> u64 {
        let segment = self.get_segment(address).unwrap();
        if !segment.readable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        return (segment.data[addr] as u64) 
        | ((segment.data[addr + 1] as u64) << 8) 
        | ((segment.data[addr + 2] as u64) << 16)
        | ((segment.data[addr + 3] as u64) << 24)
        | ((segment.data[addr + 4] as u64) << 32)
        | ((segment.data[addr + 5] as u64) << 40)
        | ((segment.data[addr + 6] as u64) << 48)
        | ((segment.data[addr + 7] as u64) << 56);
    }

    pub fn read_word(&mut self, address:u64) -> u64 {
        let segment = self.get_segment(address).unwrap();
        if !segment.readable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        return (segment.data[addr] as u64) 
        | ((segment.data[addr + 1] as u64) << 8) 
        | ((segment.data[addr + 2] as u64) << 16)
        | ((segment.data[addr + 3] as u64) << 24)
    }

    pub fn read_half(&mut self, address:u64) -> u64 {
        let segment = self.get_segment(address).unwrap();
        if !segment.readable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        return (segment.data[addr] as u64) 
        | ((segment.data[addr + 1] as u64) << 8) 
    }

    pub fn read_byte(&mut self, address:u64) -> u64 {
        let segment = self.get_segment(address).unwrap();
        if !segment.readable() {
            todo!("LOG INVALID MEMORY PERM ACCESS")
        }
        let addr = address.wrapping_sub(segment.base_address) as usize;
        return segment.data[addr] as u64;
    }

    pub fn alloc(&mut self, base_address: u64, size: usize,readable:bool,writeable:bool,executable:bool) -> u64 {
        // TODO FORCE ALIGNMENT
        let segment_base;
        let mut seg: Segment;
        if base_address != 0 {
            segment_base = base_address;
            seg = Segment{
                base_address:segment_base,
                data: vec![0;size],
                data_size: size,
                dirty:false,
                perms: vec![Permissions::NONE;0],
                readable:readable,
                writable:writeable,
                executable:executable,
            };
        } else {
            segment_base = self.next_alloc_base;
            seg = Segment{
                base_address:segment_base,
                data: vec![0;size],
                data_size: size,
                dirty:false,
                perms: vec![Permissions::NONE;0],
                readable:readable,
                writable:writeable,
                executable:executable
            };
        }
        let key = (segment_base,segment_base.wrapping_add(size as u64));
        self.virtual_memory.insert(key, seg);
        self.next_alloc_base = key.1; 
        return segment_base;
    }

}


#[derive(Debug,Clone)]
pub struct Segment {
    pub base_address: u64,
    pub data: Vec<u8>,
    pub data_size:usize,
    pub dirty: bool,
    pub executable: bool,
    pub writable: bool,
    pub readable: bool,
    pub perms: Vec<Permissions>
}

impl Segment {
    fn new() -> Segment {
        Segment{
                base_address:0,
                data: vec![0;0],
                data_size: 0,
                dirty:false,
                readable:false,
                writable:false,
                executable:false,
                perms: vec![Permissions::NONE;0],
        }
    }
    pub fn executable(&self) -> bool {
        self.executable
    }
    pub fn readable(&self) -> bool {
        self.readable
    }
    pub fn writable(&self) -> bool {
        self.writable
    }
    pub fn dirty(&self) -> bool {
        self.dirty
    }
}


#[derive(Debug,Clone)]
enum Permissions {
    READ,
    WRITE,
    EXECUTE,
    NONE,
    READWRITE,
    READEXECUTE,
}

#[derive(Debug,Clone)]
pub struct ElfSection{
    pub raw_data: Vec<u8>,
    pub raw_data_size: u64,
    pub virtual_address: u64,
    pub virtual_memory_size: u64,
    pub perms: u64,
    pub alignment: u64,
}