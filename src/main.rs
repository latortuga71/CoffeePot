
use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod data;
mod tests;

fn main() {
    //let path = "test_relaxed.elf";
    let path = "test_new";
    let mut emulator = Emulator::new();
    println!("=== CoffeePot Loading {}!  ===",path);
    //////
    let elf_segments = loader::load_elf(&path,false);
    //emulator.load_elf_segments(&elf_segments); old
    emulator.load_elf_segments_into_mmu(&elf_segments);
    emulator.cpu.pc = elf_segments.entry_point;
    emulator.cpu.mmu.alloc(0xFFFFFFF,0x1024);
    emulator.cpu.sp = 0xFFFFFFF + 0x1024;
    for (k,v) in emulator.cpu.mmu.virtual_memory_new.iter(){
        println!("{:#08X}\n {:#08X}",k.0,k.1);
    }
    //emulator.load_raw_instructions("./add.bin").unwrap();
    //print!("{:?}", emulator.cpu.mmu.text_segment);
    emulator.cpu.debug_flag = true;
    // example snapshot?
    println!("=== CoffeePot Elf Loading Complete!  ===",);
    println!("=== CoffeePot Init!  ===");
    loop {
        //println!("{}", emulator.cpu);
        // Fetch
        if !emulator.fetch_instruction() {
            break;
        }
        //print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        // Decode && Execute
        if emulator.execute_instruction() {
            // exit called!
            //println!("=== CoffeePot Exit! {}  ===",emulator.cpu.exit_status);
            break;
        }
        println!("=== CoffeePot Exit! {}  ===", emulator.cpu.exit_status);
    println!("=== Goodbye, CoffeePot! ===");
    }
}