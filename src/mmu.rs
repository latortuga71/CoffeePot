use std::{collections::HashMap};



#[derive(Debug,Clone)]
pub struct MMU {
    pub virtual_memory: Vec<u8>,
    pub virtual_memory_new: HashMap<(u64,u64),Segment>,
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
            virtual_memory: vec![0; RAM as usize], // 1GB of address space by default
            virtual_memory_new: HashMap::new(),
            next_alloc_base:0,
        }
    }

    pub fn print_segments(&self) {
        for (k,_segment) in &self.virtual_memory_new {
            println!("Segment -> {:#08X} {:#08X}",k.0,k.1);
        }
    }

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

    pub fn read_to_exec(&self, address:u64, size:usize) -> &[u8] {
        // TODO CHECK PERMS
        let key = self.find_segment(address);
        if key.2 == false {
            todo!("Handle Segmentation Faults! {}",address);
        }
        let k:(u64,u64) = (key.0,key.1);
        let segment = &self.virtual_memory_new[&k];
        if segment.executable == false {
            panic!("NON EXECUTABLE MEMORY ADDRESS {:#08X}",address);
        }
        let virtual_address =  address.wrapping_sub(segment.base_address);
        let start = virtual_address as usize;
        let end = start + size;
        println!("Virtual {:#08X} Asking {:#08X} Base {:#08X}",virtual_address,address,segment.base_address);
        //println!("{start}");
        let slice = &self.virtual_memory_new[&k].data[start..end];
        return slice;
    }

    pub fn read(&self, address:u64, size:usize) -> &[u8] {
        // TODO CHECK PERMS
        let key = self.find_segment(address);
        if key.2 == false {
            todo!("Handle Segmentation Faults! {}",address);
        }
        let k:(u64,u64) = (key.0,key.1);
        let segment = &self.virtual_memory_new[&k];
        let virtual_address =  address.wrapping_sub(segment.base_address);
        let start = virtual_address as usize;
        let end = start + size;
        println!("Virtual {:#08X} Asking {:#08X} Base {:#08X}",virtual_address,address,segment.base_address);
        //println!("{start}");
        let slice = &self.virtual_memory_new[&k].data[start..end];
        return slice;
    }
    pub fn memset(&mut self, address:u64, data:u8, size:usize) -> usize {
        // TODO CHECK PERMS
        let key = self.find_segment(address);
        if key.2 == false {
            todo!("Handle Segmentation Faults! {}",address);
        }
        let k:(u64,u64) = (key.0,key.1);
        let segment = &self.virtual_memory_new[&k];
        let virtual_address =  address.wrapping_sub(segment.base_address);
        let start = virtual_address as usize;
        let end = start + size;
        //println!("Virtual {:#08X} Asking {:#08X} Base {:#08X}",virtual_address,address,segment.base_address);
        //println!("{:#08X}",start);
        match size {
            0x1 => {
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end].fill(data);
                return 1;
            }
            0x2 => {
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end].fill(data);
                return 2;
            }
            0x4 => {
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end].fill(data);
                return 4;
            }
            0x8 => {
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end].fill(data);
                return 8;
            
            }
            _ => panic!("MMU Invalid Write Size")
        }
    }
    
    pub fn write(&mut self, address:u64,data:u64, size:usize) -> usize {
        // TODO CHECK PERMS
        let key = self.find_segment(address);
        if key.2 == false {
            todo!("Handle Segmentation Faults! {}",address);
        }
        let k:(u64,u64) = (key.0,key.1);
        let segment = &self.virtual_memory_new[&k];
        let virtual_address =  address.wrapping_sub(segment.base_address);
        let start = virtual_address as usize;
        let end = start + size;
        //println!("Virtual {:#08X} Asking {:#08X} Base {:#08X}",virtual_address,address,segment.base_address);
        //println!("{:#08X}",start);
        match size {
            0x1 => {
                let value_as_bytes = (data as u8).to_le_bytes();
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end]
                .copy_from_slice(&value_as_bytes);
                return 1;
            }
            0x2 => {
                let value_as_bytes = (data as u16).to_le_bytes();
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end]
                .copy_from_slice(&value_as_bytes);
                return 2;
            }
            0x4 => {
                let value_as_bytes = (data as u32).to_le_bytes();
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end]
                .copy_from_slice(&value_as_bytes);
                return 4;
            }
            0x8 => {
                let value_as_bytes = (data as u64).to_le_bytes();
                self.virtual_memory_new.get_mut(&k).unwrap().data[start..end]
                .copy_from_slice(&value_as_bytes);
                return 8;
            
            }
            _ => panic!("MMU Invalid Write Size")
        }
    }

    pub fn alloc(&mut self, base_address: u64, size: usize) -> u64 {
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
        return segment_base;
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