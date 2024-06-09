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
#include <assert.h>
#include <unistd.h>
#include <sys/uio.h>
#include "cpu.h"
#include "mmu.h"
#include "loader.h"
#include "hash.h"
#include "coverage.h"


typedef struct emulator_t {
    CPU cpu;
    MMU mmu;
    CoverageMap coverage;
} Emulator;

Emulator* new_emulator();
void free_emulator(Emulator* emu);

static Emulator* clone_emulator(Emulator* og);

// Coverage Callback
bool generic_record_coverage(CoverageMap* coverage,uint64_t src, uint64_t dst);

// SnapShot Restore Functions
Emulator* SnapshotVM(Emulator* emu);
void RestoreVM(Emulator* snapshot, Emulator* current);



// MMU PRIMITIVES //
void vm_print(MMU*);
bool vm_range_exists(MMU*,uint64_t address);
uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size,uint32_t perms);
void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst);
Segment* vm_get_segment(MMU* mmu, uint64_t address);
static void copy_mmu_segments(Emulator* original,Emulator* snapshot);

// MMU WRITE TO MEMORY //
void vm_write_double_word(MMU*, uint64_t address, uint64_t value);
// MMU READ FROM MEMORY //
uint64_t vm_read_double_word(MMU* mmu, uint64_t address);
uint64_t vm_read_word(MMU* mmu, uint64_t address);
uint64_t vm_read_byte(MMU* mmu, uint64_t address);
char* vm_read_string(MMU* mmu,uint64_t address);
void vm_write_string(MMU* mmu,uint64_t address, char* string);


// Emulator //
void print_registers(Emulator*);
// load elf segments into memory
void load_code_segments_into_virtual_memory(Emulator* ,CodeSegments*);
// load libc stack args into stack memory
uint64_t init_stack_virtual_memory(Emulator* emu,int argc, char** argv);
uint32_t fetch(Emulator* emu);

void execute_instruction(Emulator* emu, uint64_t instruction,coverage_callback coverage_function);
static void execute(Emulator* emu, uint64_t instruction,coverage_callback coverage_function);
static void execute_compressed(Emulator* emu, uint64_t instruction,coverage_callback coverage_function);

void emulate_syscall(Emulator* emu);

// CPU INSTRUCTIONS


#endif