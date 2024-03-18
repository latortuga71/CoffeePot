use std::{io::BufRead, process::exit};

use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod tests;

fn main() {
    let mut emulator = Emulator::new();
    /*
        let elf_segments = loader::load_elf("basic.bin");
        emulator.load_elf_segments(&elf_segments);
        let i = elf_segments.entry_point as usize;
        let iend = elf_segments.entry_point as usize + 4;
        println!("Entry Point! {:#08X}", elf_segments.entry_point);
        println!(
            "Entry Point Data ! {:?}",
            &emulator.cpu.mmu.virtual_memory[i..iend]
        );
        emulator.cpu.pc = elf_segments.entry_point;
        emulator.cpu.mmu.text_segment = emulator.cpu.mmu.virtual_memory.clone();
        println!("TODO this is hilarously long after clone");
        // for now we just clone for testing
        // https://notes.eatonphil.com/emulating-amd64-starting-with-elf.html
    // */
    emulator.load_raw_instructions("./basic.bin").unwrap();
    emulator.cpu.mmu.text_segment = emulator.cpu.mmu.virtual_memory.clone();
    print!("{:?}", emulator.cpu.mmu.text_segment);
    println!("=== CoffeePot Init!  ===");
    loop {
        //println!("{}", emulator.cpu);
        // Fetch
        if !emulator.fetch_instruction() {
            break;
        }
        // Decode && Execute
        emulator.execute_instruction();
        //print!("CoffeePot: \n{}\n", emulator.cpu);
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        print!("CoffeePot Registers: \n{}\n", emulator.cpu);
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
