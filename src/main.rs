
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
    let elf_segments = loader::load_elf(&path,false);
    emulator.load_elf_segments(&elf_segments);
    emulator.cpu.pc = elf_segments.entry_point;
    //emulator.load_raw_instructions("./add.bin").unwrap();
    //print!("{:?}", emulator.cpu.mmu.text_segment);
    let mut counter = 0;
    let mut executions = 0;
    emulator.cpu.debug_flag = false;
    // example snapshot?
    let snapshot = emulator.clone();
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
            executions += 1;
            // reset emulator
            emulator = snapshot.clone();
            println!("{} Executions",executions);
            if executions > 15 {
                break;
            }
            continue;
        }
        //print!("CoffeePot: \n{}\n", emulator.cpu);
        /*
        if emulator.cpu.pc == 0x012068 {
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
