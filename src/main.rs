use std::io::BufRead;

use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod mmu;
mod tests;

fn main() {
    println!("=== CoffeePot Init!  ===");
    println!("=== R TYPE INSTRUCTIONS COMPLETE!  ===");
    println!("=== I TYPE LOAD INSTRUCTIONS COMPLETE!  ===");
    println!("=== S TYPE STORE INSTRUCTIONS COMPLETE!  ===");
    println!("=== B TYPE BRANCH INSTRUCTIONS COMPLETE!  ===");
    println!("=== U TYPE INSTRUCTIONS COMPLETE!  ===");
    println!("=== J TYPE INSTRUCTIONS COMPLETE!  ===");
    println!("=== Missing rest of I TYPES ==");

    let mut emulator = Emulator::new();
    emulator.load_raw_instructions("./strlen.bin").unwrap();
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
        print!(
            "CoffeePot Memory: \n{:?}\n",
            &emulator.cpu.mmu.memory_segment[0xF9B..0xFFF]
        );
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
