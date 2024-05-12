#include "emulator.h"



Emulator* new_emulator(){
    Emulator* emu = (Emulator*)calloc(1,sizeof(Emulator));
    emu->mmu.next_allocation_base = 0;
    emu->mmu.virtual_memory = (Segment*)calloc(10,sizeof(Segment));
    emu->mmu.segment_count = 0;
    emu->mmu.segment_capacity = 10;
    return emu;
}

void free_emulator(Emulator* emu){
    for (int i = 0; i < emu->mmu.segment_count; i++){
        free(emu->mmu.virtual_memory[i].data);
    }
    free(emu->mmu.virtual_memory);
    free(emu);
}

void vm_print(MMU* mmu){
    for (int i = 0; i < mmu->segment_count; i++){
        fprintf(stderr,"[%d] DEBUG SEGMENT: 0x%x - 0x%x sz 0x%0x perms 0x%x\n",i,mmu->virtual_memory[i].range.start,mmu->virtual_memory[i].range.end,mmu->virtual_memory[i].data_size,mmu->virtual_memory[i].perms);
    }
}

Segment* vm_get_segment(MMU* mmu, uint64_t address){
    fprintf(stderr,"DEBUG: GETTING SEGMENT 0x%x\n",address);
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return &mmu->virtual_memory[i];
        }
    }
    return NULL;
}

bool vm_range_exists(MMU* mmu, uint64_t address){
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return true;
        }
    }
    return false;
}

uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size, uint32_t perms) {
    if (base_address == 0){
        uint64_t base = mmu->next_allocation_base + 0x1024;
        uint64_t end = base + size;
        fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base);
        if (mmu->segment_count + 1 > mmu->segment_capacity){
            // realloc
            assert("TODO! REALLOC HANDLER HERE" == 0);
        }
        mmu->virtual_memory[mmu->segment_count].range.start = base;
        mmu->virtual_memory[mmu->segment_count].range.end = end;
        mmu->virtual_memory[mmu->segment_count].data_size = size;
        mmu->virtual_memory[mmu->segment_count].data = (uint8_t*)calloc(1,size);
        mmu->virtual_memory[mmu->segment_count].perms = perms;
        printf("AND 0x%x\n",mmu->virtual_memory[mmu->segment_count].perms & WRITE);
        mmu->segment_count++;
        mmu->next_allocation_base = end;
        return base;
    }

    if (vm_range_exists(mmu, base_address)){
        fprintf(stderr, "ERROR FAILED TO ALLOCATE MEMORY RANGE TAKEN 0x%x\n",base_address);
        return -1;
    }
    if (mmu->segment_count + 1 > mmu->segment_capacity){
        // realloc
        assert("TODO! REALLOC HANDLER HERE" == 0);
    }
    uint64_t base = base_address;
    uint64_t end = base + size;
    fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base);
    mmu->virtual_memory[mmu->segment_count].range.start = base;
    mmu->virtual_memory[mmu->segment_count].range.end = end;
    mmu->virtual_memory[mmu->segment_count].data_size = size;
    mmu->virtual_memory[mmu->segment_count].data = (uint8_t*)calloc(1,size);
    mmu->virtual_memory[mmu->segment_count].perms = perms;
    printf("AND 0x%x\n",mmu->virtual_memory[mmu->segment_count].perms & WRITE);
    mmu->segment_count++;
    mmu->next_allocation_base = end;
    return base;
}

void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst) {
    Segment* segment = vm_get_segment(mmu,dst);
    if (segment == NULL){
        assert("TODO! LOG SEGFAULTS" == 0);
        return;
    }
    printf("perms 0x%x perms ord 0x%d\n",segment->perms,(segment->perms & WRITE));
    if ((segment->perms & WRITE) == 0){
        assert("TODO! LOG SEGFAULT NO WRITE PERM" == 0);
    }
    uint64_t offset = dst - segment->range.start;
    if (src_size > segment->data_size){
        assert("TODO! LOG SEGFAULT OOB" == 0);
    }
    memcpy(&segment->data[offset],src, src_size);
}

void load_code_segments_into_virtual_memory(Emulator* emu ,CodeSegment* code){
  uint64_t text_section_base = vm_alloc(&emu->mmu, code->base_address,code->total_size, READ|WRITE|EXEC);
  vm_copy(&emu->mmu,code->raw_data,code->total_size,code->base_address);
}

void init_stack_virtual_memory(Emulator* emu){
  uint64_t stack_base = vm_alloc(&emu->mmu, 0, 1024*1024, READ | WRITE);
}