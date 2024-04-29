use std::{collections::HashMap};



#[derive(Debug,Clone)]
pub struct MMU {
    pub virtual_memory: Vec<u8>,
    //pub virtual_memory_new: HashMap<(u64,u64),Segment>,
    pub virtual_memory_new: Vec<u8>,
    pub next_alloc_base: u64
}

pub const RAM: u64 = 1024 * 1024 * 1024;


pub const BYTE:usize = 0x1;
pub const HALF:usize = 0x2;
pub const WORD:usize = 0x4;
pub const DOUBLE_WORD:usize = 0x8;



impl MMU {
    pub fn new() -> Self {
        MMU {
            virtual_memory: vec![0;0], // 1GB of address space by default
            //virtual_memory_new: HashMap::new(),
            virtual_memory_new: vec![0; RAM as usize], // 1GB of address space by default
            next_alloc_base:0,
        }
    }
    /*
    pub fn print_segments(&self) {
        for (k,_segment) in &self.virtual_memory_new {
            println!("Segment -> {:#08X} {:#08X}",k.0,k.1);
        }
    }
    */
    /*
    fn find_segment(&self,address:u64) -> (u64,u64,bool){
        let mut key:(u64,u64,bool) = (0,0,false);
        println!("Attempting to find {:#08X}",address);
        for (k,_segment) in &self.virtual_memory_new {
            if address >= k.0 && address <= k.1 {
                key.0 = k.0;
                key.1 = k.1;
                key.2 = true;
                break;
            }
        }
        return key;
    }
    */

    pub fn write_byte(&mut self, address:u64,value:u64){
        self.virtual_memory_new[address as usize] = value as u8;
    }
    pub fn write_half(&mut self, address:u64,value:u64){
        let addr = address as usize;
        self.virtual_memory_new[addr] = (value & 0xff) as u8;
        self.virtual_memory_new[addr + 1] = ((value >> 8) & 0xFF) as u8;
    }

    pub fn write_word(&mut self, address:u64,value:u64){
        let addr = address as usize;
        self.virtual_memory_new[addr] = (value & 0xff) as u8;
        self.virtual_memory_new[addr + 1] = ((value >> 8) & 0xff) as u8;
        self.virtual_memory_new[addr + 2] = ((value >> 16) & 0xff) as u8;
        self.virtual_memory_new[addr + 3] = ((value >> 24) & 0xff) as u8;
    }

    pub fn write_double_word(&mut self, address:u64,value:u64){
        let addr = address as usize;
        self.virtual_memory_new[addr] = (value & 0xff) as u8;
        self.virtual_memory_new[addr + 1] = ((value >> 8) & 0xff) as u8;
        self.virtual_memory_new[addr + 2] = ((value >> 16) & 0xff) as u8;
        self.virtual_memory_new[addr + 3] = ((value >> 24) & 0xff) as u8;
        self.virtual_memory_new[addr + 4] = ((value >> 32) & 0xff) as u8;
        self.virtual_memory_new[addr + 5] = ((value >> 40) & 0xff) as u8;
        self.virtual_memory_new[addr + 6] = ((value >> 48) & 0xff) as u8;
        self.virtual_memory_new[addr + 7] = ((value >> 56) & 0xff) as u8;
    }

    pub fn read_byte(&self, address:u64) -> u64 {
        self.virtual_memory_new[address as usize] as u64
    }

    pub fn read_half(&self, address:u64) -> u64 {
        let addr = address as usize;
        return (self.virtual_memory_new[addr] as u64) | ((self.virtual_memory_new[addr + 1] as u64) << 8);
    }
    pub fn read_word(&self, address:u64) -> u64 {
        let addr = address as usize;
        return (self.virtual_memory_new[addr] as u64) 
        | ((self.virtual_memory_new[addr + 1] as u64) << 8) 
        | ((self.virtual_memory_new[addr + 2] as u64) << 16)
        | ((self.virtual_memory_new[addr + 3] as u64) << 24)
    }

    pub fn read_double_word(&self, address:u64) -> u64 {
        let addr = address as usize;
        return (self.virtual_memory_new[addr] as u64) 
        | ((self.virtual_memory_new[addr + 1] as u64) << 8) 
        | ((self.virtual_memory_new[addr + 2] as u64) << 16)
        | ((self.virtual_memory_new[addr + 3] as u64) << 24)
        | ((self.virtual_memory_new[addr + 4] as u64) << 32)
        | ((self.virtual_memory_new[addr + 5] as u64) << 40)
        | ((self.virtual_memory_new[addr + 6] as u64) << 48)
        | ((self.virtual_memory_new[addr + 7] as u64) << 56);
    }

    pub fn alloc(&mut self, base_address: u64, size: usize) -> u64 {
        /*
        // TODO! Find Unused Base Address (use next base)
        // TODO! Permissions for bytes dirty bits for segments
        println!("Attempting to alloc {:#08X} to {:#08X}",base_address,base_address.wrapping_add(size as u64));
        let inuse = self.find_segment(base_address);
        if inuse.2 {
            todo!(" HANDLE Address already in use");
        }
        let segment_base = base_address;
        let segment = Segment{
            base_address:segment_base,
            data: vec![0;size],
            data_size:size,
            grows_up: false,
            dirty:false,
            executable:true,
            perms: vec![0;size],
        };
        let key = (segment_base,segment_base.wrapping_add(size as u64));
        self.virtual_memory_new.insert(key, segment);
        */
        return base_address;
    }

}


#[derive(Debug,Clone)]
pub struct Segment {
    pub base_address: u64,
    pub data: Vec<u8>,
    pub data_size:usize,
    pub grows_up: bool,
    pub dirty: bool,
    pub executable:bool,
    pub perms: Vec<u8>
}


// todo in the future
enum Permisssions {
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