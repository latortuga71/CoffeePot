
use std::{clone, io::BufRead, time::Duration};

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
    // create thread for monitoring cases/crashes/etc{}
    let iterations = std::sync::Arc::new(std::sync::Mutex::new(0.0));
    let iter_reader = std::sync::Arc::clone(&iterations);
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let mut last_time = std::time::Instant::now();
        loop {
            let elapsed = start.elapsed().as_secs_f64();
            let count = iter_reader.lock().unwrap();
            if *count % 10000.0 == 0.0 {
                println!("{:10} iterations {:4} cases per second",count, *count / elapsed);
            }
        }

    });
    // create threads per core
    let cores = 8;
    for threads in 0..cores {
        let emulator_clone = emulator.snapshot();
        let iterations = iterations.clone();
        std::thread::spawn(move || {
            fuzz(emulator_clone, threads, iterations);
        });
    }
    loop {
        std::thread::sleep(Duration::from_millis(5000));
    }
}


fn fuzz(mut emulator: Emulator,thread_id:i32, iterations:std::sync::Arc<std::sync::Mutex<f64>>) {
    let mut base_state = emulator.snapshot();
    let mut snapshot_taken = true;
    let mut debug = false;
    loop {
        /* 
        if emulator.cpu.pc == 0x10274 && !snapshot_taken {
            base_state = emulator.snapshot(); // snapshot at MAIN
            snapshot_taken = true;
            //println!("TOOK SNAPSHOT AT {:#08X}",emulator.cpu.pc);
        }
        */
        if !emulator.fetch_instruction() {
            break;
        }
        if debug {
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        }
        /*
        if emulator.cpu.pc == 0x1029A && snapshot_taken {
            emulator.restore(&base_state);
            let mut c = iterations.lock().unwrap();
            *c += 1.0;
            //println!("RESTORED!");
            //println!("RUNNING AGAIN? -> {:#08X}",emulator.cpu.pc);
        }
        */
        if emulator.execute_instruction() {
            emulator.restore(&base_state);
            let mut c = iterations.lock().unwrap();
            *c += 1.0;
            //println!("Exiting.\n");
        }
    }
}