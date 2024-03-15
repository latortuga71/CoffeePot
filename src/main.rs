use std::{io::BufRead, process::exit};

use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod tests;

fn main() {
    loader::load_elf("test.elf");
    exit(0);
    // https://notes.eatonphil.com/emulating-amd64-starting-with-elf.html
    println!("=== CoffeePot Init!  ===");
    let mut emulator = Emulator::new();
    emulator.load_raw_instructions("./test.bin").unwrap();
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
        /*
        print!("CoffeePot Registers: \n{}\n", emulator.cpu);
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
