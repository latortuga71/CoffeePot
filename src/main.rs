use std::io::BufRead;

use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod tests;

fn main() {
    let mut emulator = Emulator::new();
    let elf_segments = loader::load_elf("test_relaxed.elf");
    emulator.load_elf_segments(&elf_segments);
    emulator.cpu.pc = elf_segments.entry_point;
    //emulator.load_raw_instructions("./add.bin").unwrap();
    //print!("{:?}", emulator.cpu.mmu.text_segment);
    let mut counter = 0;
    let mut debug = false;
    println!("=== CoffeePot Init!  ===");
    loop {
        //println!("{}", emulator.cpu);
        // Fetch
        if !emulator.fetch_instruction() {
            break;
        }
        print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        // Decode && Execute
        emulator.execute_instruction();
        //print!("CoffeePot: \n{}\n", emulator.cpu);
        /*
        if emulator.cpu.pc == 0x12220 {
            debug = true;
        }
        if debug {
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
        }
        */
        //print!("coffeepot registers: \n{}\n", emulator.cpu);
        //
        //let stdin = std::io::stdin();
        //let mut line = String::new();
        //stdin.lock().read_line(&mut line).unwrap();
        //print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        //print!("PC: {:#08X}\n", emulator.cpu.pc);
        //print!("SP: {:#08X}\n", emulator.cpu.sp);
        //print!("Instructions Executed: {}\n", counter);
        counter += 1;
        /*
                print!(
                    "CoffeePot Memory: \n{:?}\n",
                    &emulator.cpu.mmu.memory_segment[0xF9B..0xFFF]
                );
        */
        /*
                if line.contains("r") {
                    print!("CoffeePot: \n{}\n", emulator.cpu);
                }
                if line.contains("m") {
                    print!(
                        "CoffeePot: \n{:?}\n",
                        &emulator.cpu.mmu.memory_segment[0..100]
                    );
                }
        */
    }
    println!("=== Goodbye, CoffeePot! ===");
}
