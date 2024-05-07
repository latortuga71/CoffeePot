
use std::{clone, env, io::BufRead, time::Duration};

use crate::{emulator::Emulator};

mod cpu;
mod emulator;
mod loader;
mod mmu;
mod data;
mod tests;

fn main() {
    let path = "test";
    let mut emulator = Emulator::new();
    println!("=== CoffeePot Loading {}!  ===",path);
    let elf_segments = loader::load_elf(&path,false);
    emulator.load_elf_segments(&elf_segments);
    // Initalize Stack And Set Stack Pointer Register
    let argv:Vec<String> = env::args().rev().collect();
    emulator.cpu.x_reg[2]  = emulator.initialize_stack_libc(argv.len() as u64 ,argv);
    emulator.cpu.pc = elf_segments.entry_point;
    emulator.cpu.call_stack.push(emulator.cpu.pc);
    emulator.cpu.debug_flag = false;
    println!("=== CoffeePot Elf Loading Complete!  ===",);
    let mut debug = false;
    // create thread for monitoring cases/crashes/etc{}
    let iterations = std::sync::Arc::new(std::sync::Mutex::new(0.0));
    let iter_reader = std::sync::Arc::clone(&iterations);
    /*
    std::thread::spawn(move || {
        let start = std::time::Instant::now();
        let mut last_time = std::time::Instant::now();
        loop {
            let elapsed = start.elapsed().as_secs_f64();
            let count = iter_reader.lock().unwrap();
            if *count % 10000.0 == 0.0 {
                println!("{:10} iterations {:8} cases per second",count, *count / elapsed);
            }
        }

    });
    */
    // create threads per core
    let cores = 1;
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
    let mut snapshot_taken = false;
    let debug = true;
    emulator.cpu.debug_flag = true;
    loop {
        if !emulator.fetch_instruction() {
            println!("fetch failed pc => {:#08X} {:08X}",emulator.cpu.pc,emulator.current_instruction);
            break;
        }
        if emulator.cpu.pc == 0x10236 && snapshot_taken == false {
            base_state = emulator.snapshot();
            snapshot_taken = true;
        }
        if debug {
            let stdin = std::io::stdin();
            let mut line = String::new();
            stdin.lock().read_line(&mut line).unwrap();
            print!("CoffeePot Registers: \n{}\n", emulator.cpu);
        }
        if emulator.execute_instruction() {
            emulator.restore(&base_state);
            let mut c = iterations.lock().unwrap();
            *c += 1.0;
        }
    }
}