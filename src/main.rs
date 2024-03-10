use std::io::BufRead;

use crate::emulator::Emulator;

mod cpu;
mod emulator;
mod mmu;
mod tests;

fn main() {
    println!("=== CoffeePot Init!  ===");
    let mut emulator = Emulator::new();
    emulator.load_raw_instructions("./test.bin").unwrap();
    loop {
        // Fetch
        if !emulator.fetch_instruction() {
            break;
        }
        // Decode && Execute
        emulator.execute_instruction();
        //print!("CoffeePot: \n{}\n", emulator.cpu);
        /*
        let stdin = std::io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line).unwrap();
        */
    }
    println!("=== Goodbye, CoffeePot! ===");
}
