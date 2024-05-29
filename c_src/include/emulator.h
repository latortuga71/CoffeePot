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
#include "cpu.h"
#include "mmu.h"
#include "loader.h"


typedef struct emulator_t {
    CPU cpu;
    MMU mmu;
} Emulator;

Emulator* new_emulator();
void free_emulator(Emulator* emu);


// MMU PRIMITIVES //
void vm_print(MMU*);
bool vm_range_exists(MMU*,uint64_t address);
uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size,uint32_t perms);
void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst);
Segment* vm_get_segment(MMU* mmu, uint64_t address);

// MMU WRITE TO MEMORY //
void vm_write_double_word(MMU*, uint64_t address, uint64_t value);
// MMU READ FROM MEMORY //
uint64_t vm_read_double_word(MMU* mmu, uint64_t address);
uint64_t vm_read_word(MMU* mmu, uint64_t address);






// Emulator //
void print_registers(Emulator*);
// load elf segments into memory
void load_code_segments_into_virtual_memory(Emulator* ,CodeSegment*);
// load libc stack args into stack memory
uint64_t init_stack_virtual_memory(Emulator* emu,int argc, char** argv);
uint32_t fetch(Emulator* emu);
void execute_instruction(Emulator* emu, uint64_t instruction);
///void static execute(Emulator* emu);
//void static execute_compressed(Emulator* emu);

static void execute(Emulator* emu, uint64_t instruction);
static void execute_compressed(Emulator* emu, uint64_t instruction);

// CPU INSTRUCTIONS


#endif