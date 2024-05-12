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
        free(emu->mmu.virtual_memory->data);
    }
    free(emu->mmu.virtual_memory);
    free(emu);
}

void vm_print(MMU* mmu){
    for (int i = 0; i < mmu->segment_count; i++){
        fprintf(stderr,"DEBUG SEGMENT: 0x%x - 0x%x sz %d\n",mmu->virtual_memory[i].range.start,mmu->virtual_memory[i].range.end,mmu->virtual_memory->data_size);
    }
}

Segment* vm_get_segment(MMU* mmu, uint64_t address){
    fprintf(stderr,"DEBUG: GETTING SEGMENT 0x%x\n",address);
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return mmu->virtual_memory;
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

uint64_t vm_alloc(MMU* mmu, uint64_t base_address, size_t size) {
    fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base_address);
    if (base_address == 0){
        uint64_t base = mmu->next_allocation_base;
        uint64_t end = base + size;
        if (mmu->segment_count + 1 > mmu->segment_capacity){
            // realloc
            assert("TODO! REALLOC HANDLER HERE" == 0);
        }
        mmu->virtual_memory->range.start = base;
        mmu->virtual_memory->range.end = end;
        mmu->virtual_memory->data_size = size;
        mmu->virtual_memory->data = (uint8_t*)calloc(1,size);
        mmu->segment_count++;
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
    mmu->virtual_memory->range.start = base;
    mmu->virtual_memory->range.end = end;
    mmu->virtual_memory->data_size = size;
    mmu->virtual_memory->data = (uint8_t*)calloc(1,size);
    mmu->segment_count++;
    return base;
}

void vm_copy(MMU* mmu,char* src, size_t src_size, uint64_t dst) {
    Segment* segment = vm_get_segment(mmu,dst);
    if (segment == NULL){
        fprintf(stderr,"ERROR: SEGFAULT\n");
        return;
    }
    uint64_t offset = dst - segment->range.start;
    if (src_size > segment->data_size){
        assert("TODO! LOG SEGFAULTS" == 0);
    }
    memcpy(&segment->data[offset],src, src_size);
}

void load_code_segments_into_virtual_memory(Emulator* emu ,CodeSegment* code){
  uint64_t text_section_base = vm_alloc(&emu->mmu, code->base_address,code->total_size);
  vm_copy(&emu->mmu,code->raw_data,code->total_size,code->base_address);
}
