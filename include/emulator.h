#ifndef EMULATOR_HEADER

#define EMULATOR_HEADER 

#define DEBUG 1


#define debug_print(fmt, ...) \
            do { if (DEBUG) fprintf(stderr, fmt, __VA_ARGS__); } while (0)

#define error_print(fmt, ...) \
            do { fprintf(stderr, fmt, __VA_ARGS__); } while (0)

#define todo(fmt) assert("TODO -> " fmt == 0);
#define panic(fmt) assert("PANIC !!! ->" fmt == 0);

#include <stdlib.h>
#include <stdio.h>
#include <string.h>
#include <assert.h>
#include <unistd.h>
#include <sys/uio.h>
#include "cpu.h"
#include "mmu.h"
#include "loader.h"
#include "hash.h"
#include "coverage.h"
#include "crash.h"
#include "stats.h"
#include "corpus.h"

typedef struct emulator_t {
    CPU cpu;
    MMU mmu;
    FuzzCase* current_fuzz_case;
    Corpus* corpus;
    CoverageMap* coverage;
    CrashMap* crashes;
    Stats* stats;
    int file_descriptors[100];
    uint64_t snapshot_address;
    uint64_t restore_address;
    bool crashed;
    bool monitor_dirty_segments;
} Emulator;


Emulator* new_emulator(CoverageMap* coverage,CrashMap* crashes,Stats* stats,Corpus* corpus,uint64_t snapshot_addr,uint64_t restore_address);


void free_emulator(Emulator* emu);

static Emulator* clone_emulator(Emulator* og);

// Coverage Callback
bool generic_record_coverage(CoverageMap* coverage,uint64_t src, uint64_t dst);
// Crashes Callback
bool generic_record_crashes(CrashMap* crashes, uint64_t pc, FuzzCase* fcase);

// SnapShot Restore Functions
Emulator* snapshot_vm(Emulator* emu);
void restore_vm(Emulator* snapshot, Emulator* current);



// MMU PRIMITIVES //
void vm_print(MMU*);
bool vm_range_exists(MMU*,uint64_t address);
uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size,uint32_t perms);
void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst);
Segment* vm_get_segment(MMU* mmu, uint64_t address);
static void copy_mmu_segments(Emulator* original,Emulator* snapshot);

// MMU WRITE TO MEMORY //
void vm_write_double_word(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function);
void vm_write_word(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function);
void vm_write_byte(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function);
void vm_write_half(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function);

// MMU READ FROM MEMORY //
uint64_t vm_read_double_word(Emulator* emu, uint64_t address,crash_callback crashes_function);
uint64_t vm_read_word(Emulator* emu, uint64_t address,crash_callback crashes_function);
uint64_t vm_read_half(Emulator* emu, uint64_t address,crash_callback crashes_function);
uint64_t vm_read_byte(Emulator* emu, uint64_t address,crash_callback crashes_function);


// Stuff that i use for other reasons not really used in instruction set
char* vm_read_string(MMU* mmu,uint64_t address);
void vm_write_string(MMU* mmu,uint64_t address, char* string);
void vm_write_buffer(Emulator* emu,uint64_t address, uint8_t* data, size_t size);

// Emulator //
void print_registers(Emulator*);
// load elf segments into memory
void load_code_segments_into_virtual_memory(Emulator* ,CodeSegments*);
// load libc stack args into stack memory
uint64_t init_stack_virtual_memory(Emulator* emu, int argc, char** argv,crash_callback crashes_function);
uint32_t fetch(Emulator* emu, crash_callback crashes_function);

void execute_instruction(Emulator* emu, uint64_t instruction,coverage_callback coverage_function,crash_callback crash_function);
static void execute(Emulator* emu, uint64_t instruction,coverage_callback coverage_function,crash_callback crash_function);
static void execute_compressed(Emulator* emu, uint64_t instruction,coverage_callback coverage_function, crash_callback crash_function);

void emulate_syscall(Emulator* emu,crash_callback crash_function);

// CPU INSTRUCTIONS


#endif
