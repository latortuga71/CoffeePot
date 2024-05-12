#ifndef EMULATOR_HEADER

#define EMULATOR_HEADER 


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


// MMU PRIMITIVES
void vm_print(MMU*);
bool vm_range_exists(MMU*,uint64_t address);
uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size);
void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst);
Segment* vm_get_segment(MMU* mmu, uint64_t address);
void load_code_segments_into_virtual_memory(Emulator* ,CodeSegment*);

#endif