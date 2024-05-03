
use std::io::BufRead;

use crate::{emulator::Emulator};

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod data;
mod tests;

fn main() {
    let path = "invalid_ref_ok";
    let mut emulator = Emulator::new();
    println!("=== CoffeePot Loading {}!  ===",path);
    let elf_segments = loader::load_elf(&path,false);
    emulator.load_elf_segments(&elf_segments);
    // Initalize Stack And Set Stack Pointer Register
    emulator.cpu.x_reg[2]  = emulator.initialize_stack_libc(1u64, "AAAAAAAA".to_string());
    emulator.cpu.mmu.print_segments(); 
    emulator.cpu.pc = elf_segments.entry_point;
    emulator.cpu.debug_flag = false;
    println!("=== CoffeePot Elf Loading Complete!  ===",);
    let mut debug = false;
    // snapshotting ?
    let mut base_state: Emulator = emulator.snapshot();
    let mut snapshot_taken = false;
    let mut iterations:u64 = 0;
    loop {
        if emulator.cpu.pc == 0x10274 && !snapshot_taken {
            base_state = emulator.snapshot(); // snapshot at MAIN
            snapshot_taken = true;
            println!("TOOK SNAPSHOT AT {:#08X}",emulator.cpu.pc);
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
        }
        if !emulator.fetch_instruction() {
            break;
        }
        if debug {
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        }
        if emulator.cpu.pc == 0x1029A && snapshot_taken {
            iterations +=1;
            emulator.restore(&base_state);
            println!("{iterations} Iterations\n");
            //println!("RESTORED!");
            //println!("RUNNING AGAIN? -> {:#08X}",emulator.cpu.pc);
        }
        if emulator.execute_instruction() {
            println!("Exiting.\n");
        }
    }
}