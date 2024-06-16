#include "emulator.h"

Emulator* snapshot_vm(Emulator* emu){
    Emulator* snap = clone_emulator(emu);
    copy_mmu_segments(emu,snap);
    return snap;
}


void restore_vm(Emulator* emu,Emulator* og){
    emu->mmu.next_allocation_base = og->mmu.next_allocation_base;
    emu->mmu.segment_capacity = og->mmu.segment_capacity;
    // May have an issue if we alloc more memory after snapshot and before restore
    emu->mmu.segment_count = og->mmu.segment_count;
    // CPU 
    emu->cpu.pc = og->cpu.pc;
    emu->cpu.stack_pointer = og->cpu.stack_pointer;
    for (int i = 0; i < 32; i++){
        emu->cpu.x_reg[i] = og->cpu.x_reg[i];
    }
    // here we should only restore the stack
    // its mocking only restoring dirty memory segment
    // index 1 should be the stack segment
    Segment* segment_og = &og->mmu.virtual_memory[0];
    Segment* segment_new = &emu->mmu.virtual_memory[0];
    memcpy(&segment_new->data[0],&segment_og->data[0],segment_og->data_size);
    /*
    for (int i = 0; i < og->mmu.segment_count; i++){
        Segment* segment_og = &og->mmu.virtual_memory[i];
        Segment* segment_new = &emu->mmu.virtual_memory[i];
        segment_new->data_size = segment_og->data_size;
        segment_new->perms = segment_og->perms;
        segment_new->range = segment_og->range;
        segment_new->data = (uint8_t*)calloc(1,segment_new->data_size);
        memcpy(&segment_new->data[0],&segment_og->data[0],segment_og->data_size);
    }
    */
}


bool generic_record_crashes(CrashMap* crashes,uint64_t pc, FuzzCase* fcase){
    crashes->crashes++;
    char file_name[250];
    memset(file_name,0,250);
    snprintf(file_name,250,"./crashes/_0x%llx_crash_id_%u",pc,crashes->crashes);
    FILE* f = fopen(file_name,"w");
    fwrite(fcase->data,sizeof(uint8_t),fcase->size,f);
    fclose(f);
    return true;
}


bool generic_record_coverage(CoverageMap* coverage,uint64_t src, uint64_t dst){
    uint64_t target = src ^ dst;
    uint64_t hash = hashstring((unsigned char*)&target);
    if (coverage->hashes->count(hash)){
        return true;
    } 
    coverage->hashes->insert(hash);
    coverage->unique_branches_taken++;
    return true;
}

Emulator* new_emulator(CoverageMap* coverage,CrashMap* crashes,Stats* stats,Corpus* corpus,uint64_t snapshot_address,uint64_t restore_address){
    Emulator* emu = (Emulator*)calloc(1,sizeof(Emulator));
    emu->crashed = false;
    emu->corpus = corpus;
    emu->crashes = crashes;
    emu->coverage = coverage;
    emu->stats = stats;
    emu->stats->cases = 0;
    emu->stats->start_time = std::time(0);
    emu->coverage->hashes = new std::set<uint64_t>();
    emu->coverage->previous_unique_branches_taken = 0;
    emu->mmu.next_allocation_base = 0;
    emu->mmu.virtual_memory = (Segment*)calloc(10,sizeof(Segment));
    emu->mmu.segment_count = 0;
    emu->mmu.segment_capacity = 10;
    emu->snapshot_address = snapshot_address;
    emu->restore_address = restore_address;
    return emu;
}

static void copy_mmu_segments(Emulator* original,Emulator* snapshot){
    for (int i = 0; i < original->mmu.segment_count; i++){
        Segment* segment_og = &original->mmu.virtual_memory[i];
        Segment* segment_new = &snapshot->mmu.virtual_memory[i];
        segment_new->data_size = segment_og->data_size;
        segment_new->perms = segment_og->perms;
        segment_new->range = segment_og->range;
        segment_new->data = (uint8_t*)calloc(1,segment_new->data_size);
        memcpy(&segment_new->data[0],&segment_og->data[0],segment_og->data_size);
    }
}

static Emulator* clone_emulator(Emulator* og){
    Emulator* emu = (Emulator*)calloc(1,sizeof(Emulator));
    //
    // MMU
    emu->mmu.next_allocation_base = og->mmu.next_allocation_base;
    emu->mmu.segment_capacity = og->mmu.segment_capacity;
    emu->mmu.virtual_memory = (Segment*)calloc(og->mmu.segment_capacity,sizeof(Segment));
    emu->mmu.segment_count = og->mmu.segment_count;
    // CPU 
    emu->cpu.pc = og->cpu.pc;
    emu->cpu.stack_pointer = og->cpu.stack_pointer;
    for (int i = 0; i < 32; i++){
        emu->cpu.x_reg[i] = og->cpu.x_reg[i];
    }

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
        debug_print("[%d] DEBUG SEGMENT: 0x%llx-0x%llx size 0x%0x perms 0x%x\n",i,mmu->virtual_memory[i].range.start,mmu->virtual_memory[i].range.end,mmu->virtual_memory[i].data_size,mmu->virtual_memory[i].perms);
    }
}

Segment* vm_get_segment(MMU* mmu, uint64_t address){
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return &mmu->virtual_memory[i];
        }
    }
    return NULL;
    /*
    if ( (address >= 0x10000 ) && (address <= 0x34c50)){
        return &mmu->virtual_memory[0];
    } else if ( (address >= 4000000000 ) && (address <= 0x400001f400 )){
        return &mmu->virtual_memory[1];
    } else if ( (address >= 0x4000020424) && (address <= 0x4000020824 )){
        return &mmu->virtual_memory[2];
    } else {
        printf("0x%llx\n",address);
        assert("ERROR MEMORY ACCESS" == 0);
        return NULL;
    }
    */
    /*
    [0] DEBUG SEGMENT: 0x10000-0x34c50 size 0x24c50 perms 0x7
    [1] DEBUG SEGMENT: 0x4000000000-0x4001048510 size 0x1048510 perms 0x3
    [2] DEBUG SEGMENT: 0x4001049534-0x4001049934 size 0x400 perms 0x3

    1] DEBUG SEGMENT: 0x4000000000-0x4000001024 size 0x1024 perms 0x3
    [2] DEBUG SEGMENT: 0x4000002048-0x4000002448 size 0x400 perms 0x3
0x4000020424-0x4000020824

    */
   /*
    debug_print("DEBUG: GETTING SEGMENT 0x%llx\n",address);
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return &mmu->virtual_memory[i];
        }
    }
    //vm_print(mmu);
    return NULL;
    */
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
    uint64_t page_size = 4096;
    if (base_address == 0){
        // makre sure base is 8 byte aligned
        debug_print("TODO! Make Sure Allocations Are Properly Aligned%s","\n");
        uint64_t base = mmu->next_allocation_base + 0x1024;
        // force upper alignment
        /*
        base = (base & ~(page_size - 1)) + page_size;
        if ((base % 8) != 0 ){
            todo("Alignement issue\n");
        }
        */
        uint64_t end = base + size;
        //fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base);
        if (mmu->segment_count + 1 > mmu->segment_capacity){
            // realloc
            //mmu->virtual_memory = (Segment*)realloc(mmu->virtual_memory,)
            assert("TODO! REALLOC HANDLER HERE" == 0);
        }
        mmu->virtual_memory[mmu->segment_count].range.start = base;
        mmu->virtual_memory[mmu->segment_count].range.end = end;
        mmu->virtual_memory[mmu->segment_count].data_size = size;
        mmu->virtual_memory[mmu->segment_count].data = (uint8_t*)calloc(1,size);
        mmu->virtual_memory[mmu->segment_count].perms = perms;
        mmu->segment_count++;
        mmu->next_allocation_base = end;
        return base;
    }

    if (vm_range_exists(mmu, base_address)){
        error_print("ERROR FAILED TO ALLOCATE MEMORY RANGE TAKEN 0x%x\n",base_address);
        return -1;
    }
    if (mmu->segment_count + 1 > mmu->segment_capacity){
        // realloc
        assert("TODO! REALLOC HANDLER HERE" == 0);
    }
    uint64_t base = base_address;
    uint64_t end = base + size;
    //fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base);
    mmu->virtual_memory[mmu->segment_count].range.start = base;
    mmu->virtual_memory[mmu->segment_count].range.end = end;
    mmu->virtual_memory[mmu->segment_count].data_size = size;
    mmu->virtual_memory[mmu->segment_count].data = (uint8_t*)calloc(1,size);
    mmu->virtual_memory[mmu->segment_count].perms = perms;
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
    //printf("perms 0x%x perms ord 0x%d\n",segment->perms,(segment->perms & WRITE));
    if ((segment->perms & WRITE) == 0){
        assert("TODO! LOG SEGFAULT NO WRITE PERM" == 0);
    }
    uint64_t offset = dst - segment->range.start;
    if (src_size > segment->data_size){
        assert("TODO! LOG SEGFAULT OOB" == 0);
    }
    memcpy(&segment->data[offset],src, src_size);
}

void load_code_segments_into_virtual_memory(Emulator* emu ,CodeSegments* code){
  uint64_t base_addr = vm_alloc(&emu->mmu,code->base_address,code->total_size, READ|WRITE|EXEC );
  for (int i = 0; i < code->count; i++){
    vm_copy(&emu->mmu, code->segs[i]->raw_data, code->segs[i]->size, code->segs[i]->virtual_address);
  }
}

uint64_t init_stack_virtual_memory(Emulator* emu, int argc, char** argv, crash_callback crashes_function){
  uint64_t stack_base = 0x4000000000;
  // 0x1F400 = 128k
  uint64_t alloc_base = vm_alloc(&emu->mmu, stack_base, 0x1F400, READ | WRITE);
  debug_print("base 0x%llx\n",alloc_base);
  uint64_t stack_end = stack_base + (0x1F400);
  uint64_t stack_pointer = stack_end - (0xFA00);
  stack_pointer -= 0x8;
  vm_write_double_word(emu,stack_pointer, 0,crashes_function);

  stack_pointer -= 0x8;
  vm_write_double_word(emu,stack_pointer, 0,crashes_function);

  stack_pointer -= 0x8;
  vm_write_double_word(emu,stack_pointer, 0,crashes_function);
  /// loop over args and write them
  while (*argv != NULL){
    uint64_t string_address = vm_alloc(&emu->mmu,0 , 1024 ,READ|WRITE);
    vm_write_string(&emu->mmu,string_address,*argv);
    stack_pointer -= 0x8;
    vm_write_double_word(emu, stack_pointer, string_address,crashes_function);
    argv++;
  }
  stack_pointer -= 0x8;
  vm_write_double_word(emu,stack_pointer, argc,crashes_function);
  debug_print("0x%llx\n",stack_pointer);
  emu->cpu.stack_pointer = stack_pointer;
  emu->cpu.x_reg[2] = stack_pointer;
  return stack_pointer;
}


void vm_write_byte(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function)  {
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return;
    }
    uint64_t index = address - s->range.start;
    debug_print("WRITE BYTE Writing 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (uint8_t)(value);
}

void vm_write_half(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function)  {
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return;
    }
    uint64_t index = address - s->range.start;
    debug_print("Writing WORD 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (value & 0xff);
    s->data[index + 1] = ((value >> 8 ) & 0xff);
}

void vm_write_word(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function)  {
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return;
    }
    uint64_t index = address - s->range.start;
    debug_print("Writing WORD 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (value & 0xff);
    s->data[index + 1] = ((value >> 8 ) & 0xff);
    s->data[index + 2] = ((value >> 16 ) & 0xff);
    s->data[index + 3] = ((value >> 24 ) & 0xff);
}


void vm_write_double_word(Emulator* emu, uint64_t address, uint64_t value,crash_callback crashes_function)  {
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return;
    }
    uint64_t index = address - s->range.start;
    debug_print("Writing DOUBLE 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (value & 0xff);
    s->data[index + 1] = ((value >> 8 ) & 0xff);
    s->data[index + 2] = ((value >> 16 ) & 0xff);
    s->data[index + 3] = ((value >> 24 ) & 0xff);
    s->data[index + 4] = ((value >> 32 ) & 0xff);
    s->data[index + 5] = ((value >> 40 ) & 0xff);
    s->data[index + 6] = ((value >> 48 ) & 0xff);
    s->data[index + 7] = ((value >> 56 ) & 0xff);
}


void vm_write_string(MMU* mmu,uint64_t address, char* string){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    while (*string != NULL){
        s->data[index] = *string;
        index++;
        string++;
    }
}

void* vm_read_memory(MMU* mmu,uint64_t address) {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    return (void*)&s->data[index];
}


void vm_write_buffer(MMU* mmu,uint64_t address, uint8_t* data, size_t size){
    /*
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    */
    // we know that its in the text section so we dont need to guess the segment
    Segment* s = &mmu->virtual_memory[0];
    uint64_t index = address - s->range.start;
    memcpy(&s->data[index],data,size);
    return;
}

void* vm_copy_memory(MMU* mmu,uint64_t address,size_t count) {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    uint8_t* copy = (uint8_t*)malloc(sizeof(uint8_t) * count);
    memset(copy,0,count);
    memcpy(copy,&s->data[index], sizeof(uint8_t) * count);
    return copy;
}

char* vm_read_string(MMU* mmu,uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    return (char*)&s->data[index];
}

uint64_t vm_read_double_word(Emulator* emu, uint64_t address,crash_callback crashes_function){
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return 0;
    }
    uint64_t index = address - s->range.start;
    debug_print("READING DOUBLE 0x%llx\n",address);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8)
        | ((uint64_t)(s->data[index + 2]) << 16)
        | ((uint64_t)(s->data[index + 3]) << 24)
        | ((uint64_t)(s->data[index + 4]) << 32)
        | ((uint64_t)(s->data[index + 5]) << 40)
        | ((uint64_t)(s->data[index + 6]) << 48)
        | ((uint64_t)(s->data[index + 7]) << 56);
}

uint64_t vm_read_word(Emulator* emu, uint64_t address,crash_callback crashes_function){
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return 0;
    }
    uint64_t index = address - s->range.start;
    debug_print("READING WORD 0x%llx\n",address);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8)
        | ((uint64_t)(s->data[index + 2]) << 16)
        | ((uint64_t)(s->data[index + 3]) << 24);
}

uint64_t vm_read_half(Emulator* emu, uint64_t address,crash_callback crashes_function){
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return 0;
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    debug_print("READING HALF 0x%llx\n",address);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8);
}
uint64_t vm_read_byte(Emulator* emu, uint64_t address,crash_callback crashes_function){
    Segment* s = vm_get_segment(&emu->mmu, address);
    if (s == NULL){
        crashes_function(emu->crashes,emu->cpu.pc,emu->current_fuzz_case);
        emu->crashed = true;
        return 0;
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    debug_print("READING BYTE 0x%llx\n",address);
    return (uint64_t)(s->data[index]);
}

uint32_t fetch(Emulator* emu,crash_callback crash_function) {
    return (uint32_t)vm_read_word(emu,emu->cpu.pc,crash_function);
}

void print_registers(Emulator* emu){
    debug_print("pc: 0x%llx\n",emu->cpu.pc);
    for (int i = 0; i < 32; i++){
        debug_print("x%d 0x%llx\n",i,emu->cpu.x_reg[i]);
    }
}

void execute_instruction(Emulator* emu, uint64_t instruction, coverage_callback coverage_function,crash_callback crashes_function){
    emu->cpu.x_reg[0] = 0;
    if ((0x3 & instruction) != 0x3) {
        debug_print("DEBUG: COMPRESSED 0x%02x\n",(uint16_t)instruction);
        execute_compressed(emu, instruction,coverage_function,crashes_function);
        emu->cpu.pc += 0x2;
    } else {
        debug_print("DEBUG: 0x%08x\n", instruction);
        execute(emu,instruction,coverage_function,crashes_function);
        emu->cpu.pc += 0x4;
    }
}

static void execute_compressed(Emulator* emu, uint64_t instruction, coverage_callback coverage_function,crash_callback crashes_function){
    uint64_t opcode = instruction & 0b11;
    uint64_t funct3 = (instruction >> 13) & 0x7;
    switch (opcode)
    {
    case 0b00:{
        debug_print("DEBUG QUADRANT %d\n",0);
        switch (funct3)
        {
            case 0x0:{
                debug_print("c.addi4spn %s","\n");
                uint64_t rd = ((instruction >> 2) & 0x7) + 8;
                uint64_t nzuimm = ((instruction >> 1) & 0x3c0)
                | ((instruction >> 7) & 0x30)
                | ((instruction >> 2) & 0x8)
                | ((instruction >> 4) & 0x4);
                if (nzuimm == 0) {
                    panic("Illegal instruction");
                    return;
                }
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[2] + nzuimm;
                return;
            }
            case 0x2: {
                uint64_t rd = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0x40) // imm[6]
                            | ((instruction >> 7) & 0x38) // imm[5:3]
                            | ((instruction >> 4) & 0x4); // imm[2]
                debug_print("DEBUG c_lw x%d, %d (x%d)\n",rd,offset,rs1);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                uint64_t result = vm_read_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)result));
                return;
            }
            case 0x3: {
                uint64_t rd = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0xc0) // imm[7:6]
                            | ((instruction >> 7) & 0x38); // imm[5:3]
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                uint64_t result = vm_read_double_word(emu,memory_address,crashes_function);
                debug_print("DEBUG c_ld x%d, %d (x%d)\n",rd,offset,rs1);
                debug_print("LOADED 0x%x\n",result);
                emu->cpu.x_reg[rd] = result;
                return;
            }
            case 0x6:{
                uint64_t rs2 = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0x40) 
                | ((instruction >> 7) & 0x38)
                | ((instruction >> 4) & 0x4);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                vm_write_word(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                debug_print("c.sw x%d,%d(x%d)\n",rs2,offset,rs1);
                return;
            }
            case 0x7: {
                uint64_t rs2 = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0xc0) 
                | ((instruction >> 7) & 0x38);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                vm_write_double_word(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                debug_print("c.sd x%d,%d(x%d)\n",rs2,offset,rs1);
                return;
            }
            default: {
                assert("UNKNOWN FUNC3 QUADRANT 0" == 0);
                return;
                }
            }
    }
    case 0b01: {
        debug_print("DEBUG QUADRANT %d\n",1);
        switch (funct3) {
            case 0x0: {
                uint64_t rd = (instruction >> 7) & 0x1f;
                uint64_t nzimm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                if ((nzimm & 0x20) != 0){
                    nzimm = (uint64_t)((int64_t)((int8_t)(0xc0 | nzimm)));
                }
                debug_print("DEBUG c_addi x%d, x%d, %d \n",rd,rd, (int16_t)nzimm);
                if (rd != 0){
                    emu->cpu.x_reg[rd] = (emu->cpu.x_reg[rd] + nzimm);
                }
                return;
            }
            case 0x1: {
                uint64_t rd = (instruction >> 7) & 0x1f;
                uint64_t imm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                if ((imm & 0x20) != 0){
                    imm = (uint64_t)((int64_t)((int8_t)(0xc0 | imm)));
                }
                debug_print("DEBUG c_addiw x%d, x%d, %d \n",rd,rd, (int16_t)imm);
                if (rd != 0){
                    emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)(emu->cpu.x_reg[rd] + imm)));
                }
                return;
            }
            case 0x2: {
                uint64_t rd = (instruction >> 7) & 0x1f;
                uint64_t imm_ = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                debug_print("DEBUG c_li x%d,%x\n",rd, imm_);
                if ((imm_ & 0x20) != 0){
                    imm_ = (int64_t)((int8_t)((0xc0 | imm_)));
                }
                if (rd != 0){
                    emu->cpu.x_reg[rd] = imm_;
                }
                return;
            }
            case 0x3: {
                uint64_t rd = (instruction >> 7) & 0x1f;;
                if (rd == 0) {
                    return;
                } else if (rd == 2) {
                    uint64_t nzimm = ((instruction >> 3) & 0x200) |
                    ((instruction >> 2) & 0x10) |
                    ((instruction << 1) & 0x40) |
                    ((instruction << 4) & 0x180) |
                    ((instruction << 3) & 0x20);
                    if ((nzimm & 0x200 ) != 0 ){
                       nzimm = (uint64_t)((int64_t)((int32_t)((int16_t)(0xfc00 | nzimm))));
                    }
                    if (nzimm != 0 ){
                        emu->cpu.x_reg[2] = (emu->cpu.x_reg[2] + nzimm);
                    }
                    debug_print("c.addi16sp 0x%x 0x%x 0x%x\n",rd,rd,nzimm);
                    return;
                } else {
                    debug_print("c.lui%s","\n");
                    uint64_t nzimm = ((instruction << 5) & 0x20000) | ((instruction << 10) & 0x1f000);
                    if ((nzimm & 0x20000) != 0){
                        nzimm = (uint64_t)((int64_t)((int32_t)((0xfffc0000 | nzimm))));
                    }
                    if (nzimm != 0) {
                        emu->cpu.x_reg[rd] = nzimm;
                    }
                    return;
                }
                return;
            }
            case 0x4: {
                uint64_t funct2 = (instruction >> 10) & 0x3;
                switch (funct2)
                {
                case 0x0:{
                    debug_print("c.srli%s","\n");
                    uint64_t rd = ((instruction >> 7) & 0b111) + 8;
                    uint64_t shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rd] >> shamt;
                    return;
                }
                case 0x1: {
                    debug_print("c.srai%s","\n");
                    uint64_t rd = ((instruction >> 7) & 0b111) + 8;
                    uint64_t shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    emu->cpu.x_reg[rd] = (uint64_t)((int64_t)(emu->cpu.x_reg[rd]) >> shamt);
                    return;
                }
                case 0x2: {
                    uint64_t rd = ((instruction >> 7) & 0b111) + 8;
                    uint64_t imm = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                    if ( (imm & 0x20) != 0 ){
                        imm = (uint64_t)((int64_t)((int8_t)((0xc0 | imm))));
                    }
                    debug_print("c.andi x%d, x%d, %d\n",rd,rd,(int64_t)imm);
                    emu->cpu.x_reg[rd] = (emu->cpu.x_reg[rd] & imm);
                    return;
                }
                case 0x3: {
                    uint64_t left = ((instruction >> 12) & 0b1);
                    uint64_t right = ((instruction >> 5) & 0b11);
                    uint64_t rd =  ((instruction >> 7) & 0b111) +8;
                    uint64_t rs2 =  ((instruction >> 2) & 0b111) +8;
                    if (left == 0x0 && right == 0x0){
                        debug_print("c.sub x%d, x%d, 0x%x\n",rd,rd,rs2);
                        emu->cpu.x_reg[rd] = (emu->cpu.x_reg[rd] - emu->cpu.x_reg[rs2]);
                        return;
                    } else if (left == 0x0 && right == 0x1) {
                        todo("c.xor");
                        return;
                    } else if (left == 0x0 && right == 0x2) {
                        debug_print("c.or%s","\n");
                        emu->cpu.x_reg[rd] = emu->cpu.x_reg[rd] | emu->cpu.x_reg[rs2];
                        return;
                    } else if (left == 0x0 && right == 0x3) {
                        debug_print("c.and%s","\n");
                        emu->cpu.x_reg[rd] = emu->cpu.x_reg[rd] & emu->cpu.x_reg[rs2];
                        return;
                    } else if (left == 0x1 && right == 0x0) {
                        todo("c.subw");
                        return;
                    } else if (left == 0x1 && right == 0x1) {
                        debug_print("c.addw x%d, x%d, 0x%x\n",rd,rd,rs2);
                        emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)((emu->cpu.x_reg[rd] + emu->cpu.x_reg[rs2]))));
                        return;
                    } else {
                        todo("panic Invalid quadrant 2 funct 2");
                        return;
                    }
                }
                default:
                    todo("panic Unknown funct 2");
                }
                todo("quadrant 1 func 3 0x4");
            }
            case 0x5: {
                debug_print("c.j%s","\n");
                uint64_t offset = ((instruction >> 1) & 0x800) |
                ((instruction << 2) & 0x400) |
                ((instruction >> 1) & 0x300) |
                ((instruction << 1) & 0x80) |
                ((instruction >> 1) & 0x40) |
                ((instruction << 3) & 0x20) |
                ((instruction >> 7) & 0x10) |
                ((instruction >> 2) & 0xe);
                if ((offset & 0x800 ) != 0 ){
                    offset = (uint64_t)((int64_t)((int16_t)((0xf000 | offset))));
                }
                emu->cpu.pc = (emu->cpu.pc + offset) - 0x2;
                return;
            }
            case 0x6: {
                uint64_t rs1 = ((instruction >>7 ) & 0b111) + 8;
                uint64_t offset = ((instruction >> 4) & 0x100) |
                ((instruction << 1) & 0xc0) |
                ((instruction << 3) & 0x20) |
                ((instruction >> 7) & 0x18) |
                ((instruction >> 2) & 0x6);
                debug_print("c_beqz x%d, 0x%x\n",rs1,emu->cpu.pc + offset);
                if ((offset & 0x100 ) != 0 ){
                    offset = (uint64_t)((int64_t)((int16_t)((0xfe00 | offset))));
                }
                if (emu->cpu.x_reg[rs1] == 0) {
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + offset) - 0x2);
                    emu->cpu.pc = (emu->cpu.pc + offset) - 0x2;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + 0x2));
                }
                return;
            }
            case 0x7: {
                uint64_t rs1 = ((instruction >>7 ) & 0b111) + 8;
                uint64_t offset = ((instruction >> 4) & 0x100) |
                ((instruction << 1) & 0xc0) |
                ((instruction << 3) & 0x20) |
                ((instruction >> 7) & 0x18) |
                ((instruction >> 2) & 0x6);
                debug_print("c_bnez x%d, 0x%x\n",rs1,emu->cpu.pc + offset);
                if ((offset & 0x100 ) != 0 ){
                    offset = (uint64_t)((int64_t)((int16_t)((0xfe00 | offset))));
                }
                if (emu->cpu.x_reg[rs1] != 0) {
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + offset) - 0x2);
                    emu->cpu.pc = (emu->cpu.pc + offset) - 0x2;
                    //printf("Branch to 0x%llx taken\n",emu->cpu.pc);
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + 0x2));
                }
                return;
            }
            default:{
                debug_print("FUNCT 3 ? %d\n",funct3);
                assert("TODO QUADRANT 1 FUNCT 3" == 0 );
                return;
            }
        }
    }
    case 0b10: {
        debug_print("DEBUG QUADRANT %d\n",2);
        switch (funct3) {
            case 0x0: {
                uint64_t rd = (instruction >> 7 ) & 0x1f;
                uint64_t shamt = ((instruction >> 7) & 0x20) | ((instruction >> 2) & 0x1f);
                debug_print("DEBUG: c.slli x%d, x%d , 0x%x\n",rd,rd,shamt);
                if (rd != 0) {
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rd] << shamt;
                }
                return;
            }
            case 0x1: {
                todo("c.fldsp");
                return;
            }
            case 0x2: {
                uint64_t rd = (instruction >> 7 ) & 0x1f;
                uint64_t offset = ((instruction << 4) & 0xc0) |
                ((instruction >> 7) & 0x20) |
                ((instruction >> 2) & 0x1c);
                uint64_t memory_address = emu->cpu.x_reg[2] + offset;
                debug_print("c.lwsp%s","\n");
                uint64_t result = vm_read_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)(result)));
                return;
            }
            case 0x3: {
                uint64_t rd = (instruction >> 7 ) & 0x1f;
                uint64_t offset = ((instruction << 4) & 0x1c0) |
                ((instruction >> 7) & 0x20) |
                ((instruction >> 2) & 0x18);
                uint64_t memory_address = emu->cpu.x_reg[2] + offset;
                debug_print("c.ldsp%s","\n");
                uint64_t result = vm_read_double_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = result;
                return;
            }
            case 0x6: {
                uint64_t rs2 = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1 ) & 0x40) 
                | ((instruction >> 7) & 0x38)
                | ((instruction >> 4) & 0x4);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                debug_print("c.sw x%d, 0x%x(x%d)\n",rs2,offset,rs1);
                vm_write_word(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                return;
            }
            case 0x7: {
                uint64_t rs2 = (instruction >> 2) & 0x1f;
                uint64_t offset = ((instruction >> 1 ) & 0x1c0) | ((instruction >> 7) & 0x38);
                uint64_t memory_address = emu->cpu.x_reg[2] + offset;
                debug_print("c.sdsp x%x,0x%x(sp)\n",rs2,offset);
                vm_write_double_word(emu, memory_address, emu->cpu.x_reg[rs2],crashes_function);
                uint64_t after = vm_read_double_word(emu, memory_address,crashes_function);
                return;
            }
            case 0x4: {
                uint64_t left = (instruction >> 12) & 0x1;
                uint64_t right = (instruction >> 2) & 0x1f;
                uint64_t rd = (instruction >> 7 ) & 0x1f;
                uint64_t rs2 = (instruction >> 2 ) & 0x1f;
                if (left == 0 && right == 0){
                    debug_print("c.jr (%s)\n","ret");
                    uint64_t rs1 = (instruction >> 7) & 0x1f;
                    if (rs1 != 0) {
                        emu->cpu.pc = emu->cpu.x_reg[rs1] - 0x2;
                    }
                    return;
                } else if (left == 0 && right != 0){
                    debug_print("c.mv %s\n","quadrant 2");
                    if (rs2 != 0){
                        emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs2];
                    }
                    return;
                } else if (left == 1 && right == 0){
                    uint64_t rd = (instruction >> 7) & 0x1f;
                    if (rd == 0 ) {
                        todo("DEBUG c_ebreak");
                        return;
                    } else {
                        debug_print("c.jalr%s","\n");
                        uint64_t rs1 = (instruction >> 7) & 0x1f;
                        uint64_t t = emu->cpu.pc + 2;
                        emu->cpu.pc = emu->cpu.x_reg[rs1] - 2;
                        emu->cpu.x_reg[1] = t;
                        return;
                    }
                } else if (left == 1 && right != 0){
                    debug_print("c.add x%d, x%d, x%d\n",rd,rd,rs2);
                    if (rs2 != 0){
                        emu->cpu.x_reg[rd] = (emu->cpu.x_reg[rd] + emu->cpu.x_reg[rs2]);
                    }
                    return;
                } else {
                    assert("QUADRANT 2 INVALID INSTRUCTION" == 0 );
                    return;
                }
            }
            default:
                assert("INVALID FUNCT3 QUADRANT 2" == 0);
                return;
        }
    }
    default:
        assert("INVALID QUADRANT " == 0);
    }
}





static void execute(Emulator* emu, uint64_t instruction,coverage_callback coverage_function,crash_callback crashes_function){
    // decode get what we need
    uint64_t opcode = instruction & 0x0000007f;
    uint64_t rd  = (instruction & 0x00000f80) >> 7;
    uint64_t rs1 = (instruction & 0x000f8000) >> 15;
    uint64_t rs2 = (instruction & 0x01f00000) >> 20;
    uint64_t funct3 = (instruction & 0x00007000) >> 12;
    uint64_t funct7 = (instruction & 0xfe000000) >> 25;
    uint64_t funct5 = (funct7 & 0b1111100) >> 2;
    switch (opcode)
    {
        case 0b00101111: {
            if ((funct3 == 0x2) && (funct5 == 0x0)){
                todo("amoadd.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x0)){
                todo("amoadd.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x1)){
                todo("amoswap.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x1)){
                todo("amoswap.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x2)){
                debug_print("lr.w%s","\n");
                uint64_t memory_address = emu->cpu.x_reg[rs1];
                if ((memory_address % 4) != 0){
                    panic("unaligned address lr.w instruction");
                }
                uint64_t value = vm_read_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)(value)));
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x2)){
                todo("lr.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x3)){
                todo("sc.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x3)){
                todo("sc.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x4)){
                todo("amoxor.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x4)){
                todo("amoxor.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x8)){
                todo("amooor.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x8)){
                todo("amooor.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0xc)){
                todo("amoand.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0xc)){
                todo("amoand.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x10)){
                todo("amomin.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x10)){
                todo("amomin.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x14)){
                todo("amomax.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x14)){
                todo("amomax.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x18)){
                todo("amominu.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x18)){
                todo("amominu.d");
                return;
            } else if ((funct3 == 0x2) && (funct5 == 0x1c)){
                todo("amomaxu.w");
                return;
            } else if ((funct3 == 0x3) && (funct5 == 0x1c)){
                todo("amomaxu.d");
                return;
            } else {
                panic("invalid a extension instruction");
            }
            return;
        }
        case 0b00111011: {
            if ((funct3 == 0x0) && (funct7 == 0x00)){
                debug_print("DEBUG: addw%s","\n");
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)((emu->cpu.x_reg[rs1] + emu->cpu.x_reg[rs2]))));
                return;
            } else if ((funct3 == 0x0) && (funct7 == 0x20)) {
                debug_print("DEBUG: subw%s","\n");
                emu->cpu.x_reg[rd] = (uint64_t)((int32_t)(emu->cpu.x_reg[rs1] - emu->cpu.x_reg[rs2]));
                return;
            } else {
                todo("subw, mulw, etc etc");
                return;
            }
            return;
        }
        case 0b0110011: {
            switch (funct3)
            {
            case 0x0:
                if (funct7 == 0x0){
                    debug_print("DEBUG: add x%d x%d x%d\n",rd,rs1,rs2);
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] + emu->cpu.x_reg[rs2];
                    return;
                } else if (funct7 == 0x1){
                    debug_print("DEBUG: %d\n","mul");
                    emu->cpu.x_reg[rd] = (uint64_t)((int64_t)(emu->cpu.x_reg[rs1]) * (int64_t)(emu->cpu.x_reg[rs2]));
                    return;
                } else if (funct7 == 0x20){
                    debug_print("sub x%d, x%d\n",rd,rs2);
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] - emu->cpu.x_reg[rs2];
                    return;
                } else {
                    assert("INVALID FUNCT 7" == 0);
                    return;
                }
            case 0x4: {
                switch (funct7)
                {
                    case 0x0: {
                        todo("xor");
                        break;
                    }
                    case 0x1: {
                        todo("div");
                        break;
                    }
                    default:{
                        todo("add, xor rem sra");
                        break;
                    }
                }
            }
            case 0x3: {
                debug_print("sltu%s","\n");
                if (funct7 == 0x0){
                    if (emu->cpu.x_reg[rs1] < emu->cpu.x_reg[rs2]){
                        emu->cpu.x_reg[rd] = 1;
                    } else {
                        emu->cpu.x_reg[rd] = 0;
                    }
                    return;
                } 
                if (funct7 == 0x1){
                    todo("mulu");
                    return;
                }
                panic("unknown func7;");
                return;
            }
            case 0x6: {
                if (funct7 == 0x0){
                    debug_print("or%s","\n");
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] | emu->cpu.x_reg[rs2];
                    return;
                }
            }
            case 0x7:{
                if (funct7 == 0x0 ){
                    debug_print("and%s","\n");
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] & emu->cpu.x_reg[rs2];
                    return;
                }
                if (funct7 == 0x1){
                    todo("remu");
                    return;
                }
            }
            default:
                assert("INVALID FUNCT 3" == 0);
                return;
            }
        }
        case 0b0000011: {
            uint64_t offset = ((int64_t)((int32_t)(instruction)) >> 20);
            uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
            switch (funct3)
            {
            case 0x0: {
                todo("load byte");
                return;
            }
            case 0x1:{
                debug_print("lh%s","\n");
                uint64_t value = vm_read_half(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = (int64_t)((int16_t)(value));
                return;
            }
            case 0x2: {
                debug_print("lw%s","\n");
                uint64_t value = vm_read_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x3: {
                debug_print("ld%s","\n");
                uint64_t value = vm_read_double_word(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x4: {
                debug_print("lbu%s","\n");
                uint64_t value = vm_read_byte(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x5: {
                debug_print("lhu%s","\n");
                uint64_t value = vm_read_half(emu,memory_address,crashes_function);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x6: {
                todo("lwu");
                return;
            }
            default:
                todo("invalid load instruction");
                return;
            }
        }
        case 0b0110111: {
            uint64_t imm = (uint64_t)((int64_t)((int32_t)(instruction & 0xfffff000)));
            emu->cpu.x_reg[rd] = imm;
            debug_print("DEBUG: lui x%d,0x%01x\n",rd,imm);
            return;
        }
        case 0b0010111: {
            uint64_t imm = (uint64_t)((int64_t)((int32_t)(instruction & 0xfffff000)));
            emu->cpu.x_reg[rd] = emu->cpu.pc + imm;
            debug_print("DEBUG: auipc x%d,0x%01x\n",rd,imm);
            return;
        }
        case 0b0011011:{
            uint64_t imm = (int64_t)((int32_t)(instruction >> 20));
            switch (funct3)
            {
            case 0x0:{
                debug_print("addiw x%d, x%d, 0x%x\n",rd,rs1,imm);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((emu->cpu.x_reg[rs1] + imm)));
                return;
            }
            case 0x1: {
                uint64_t shamt = (int64_t)((int32_t)((uint32_t)((instruction & 0x1f))));
                debug_print("slliw x%d, x%d, 0x%x\n",rd,rs1,imm);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)((emu->cpu.x_reg[rs1] << shamt))));
                return;
            }
            case 0x5:{
                switch (funct7)
                {
                    case 0x20:{
                        uint32_t shamt = (uint32_t)(imm & 0x1f);
                        debug_print("sraiw x%d, x%d, 0x%x\n",rd,rs1,imm);
                        emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)(emu->cpu.x_reg[rs1]) >> shamt));
                        return;
                    }
                    default:{
                        panic("unknwon funct7");
                        break;
                    }
                }
            }
            default:
                todo("slliw unknown funct3");
                return;
            }
        }
        case 0b1100011: {
            uint64_t first = (int64_t)((int32_t)(instruction & 0x80000000)) >> 19;
            uint64_t imm = first |
            ((instruction & 0x80) << 4) |
            ((instruction >> 20) & 0x7e0) |
            ((instruction >> 7) & 0x1e);
            switch (funct3)
            {
            case 0x0: {
                debug_print("beq x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] == emu->cpu.x_reg[rs2]) {
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            case 0x1: {
                debug_print("bne x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] != emu->cpu.x_reg[rs2]){
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            case 0x4: {
                // TODO: FOR SOME REASON int64_t doesnt get interpreted as less than zero when a negative number
                debug_print("blt x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                int32_t left = emu->cpu.x_reg[rs1];
                int32_t right = emu->cpu.x_reg[rs2];
                if (left < right){
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            case 0x5: {
                debug_print("bge x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                int32_t left = emu->cpu.x_reg[rs1];
                int32_t right = emu->cpu.x_reg[rs2];
                debug_print("%d >= %d\n",left,right);
                if (left >= right){
                    debug_print("%d%s%d\n",left," is greater than ",right);
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            case 0x6: {
                debug_print("bltu x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] < emu->cpu.x_reg[rs2]){
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            case 0x7: {
                debug_print("bgeu x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] >= emu->cpu.x_reg[rs2]){
                    coverage_function(emu->coverage,emu->cpu.pc,(emu->cpu.pc + imm) - 0x4);
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                } else {
                    coverage_function(emu->coverage,emu->cpu.pc,emu->cpu.pc + 0x4); 
                }
                return;
            }
            default:
                assert("TODO BRANCH INSTRUCT" == 0);
            }
        }
        case 0b0100011: {
            uint64_t offset = (uint64_t)(((int64_t)((int32_t)((instruction & 0xfe000000))) >> 20)) 
            | ((instruction >> 7) & 0x1f);
            uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
            switch (funct3)
            {
            case 0x0:{
                debug_print("sb x%d,0x%x,x%d\n",rs2,offset,rs1);
                vm_write_byte(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                return;
            }
            case 0x1: {
                todo("sh");
                return;
            }
            case 0x2: {
                debug_print("sw x%d,0x%x,x%d\n",rs2,offset,rs1);
                vm_write_word(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                return;
            }
            case 0x3: {
                debug_print("sd x%d,%d,x%d\n",rs2,offset,rs1);
                vm_write_double_word(emu,memory_address,emu->cpu.x_reg[rs2],crashes_function);
                return;
            }
            default:
                assert("TODO STORE INSTRUCT" == 0);
                return;
            }
        }
        case 0b00010011: {
            uint64_t imm = (uint64_t)((( (int64_t)((int32_t)instruction)) >> 20));
            uint64_t funct6 = funct7 >> 1;
            switch (funct3)
            {
            case 0x0:{
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] + imm;
                debug_print("DEBUG: addi x%d,x%d, 0x%x\n",rd,rs1,imm);
                return;
            }
            case 0x1: {
                uint64_t shamt = (instruction >> 7) & 0x3f;
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] << shamt;
                debug_print("slli x%d,x%d, 0x%x\n",rd,rs1,imm);
                return;
            }
            case 0x4:{
                debug_print("xori%s","\n");
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] ^ imm;
                return;
            }
            case 0x5: {
                if (funct6 == 0x0) {
                    uint64_t shamt = (instruction >> 20) & 0x3f;
                    emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] >> shamt;
                    debug_print("srli x%d, x%d, 0x%x\n",rd,rs1,shamt);
                    return;
                } else if (funct6 == 0x10) {
                    uint64_t shamt = (instruction >> 20) & 0x3f;
                    emu->cpu.x_reg[rd] = (uint64_t)((int64_t)(emu->cpu.x_reg[rs1]) >> shamt);
                    debug_print("srai x%d, x%d, 0x%x\n",rd,rs1,shamt);
                    return;
                } else {
                    todo("Unknown funct 6");
                    return;
                }
            }
            case 0x7: {
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] & imm;
                debug_print("DEBUG: andi x%d,x%d, 0x%x\n",rd,rs1,imm);
                return;
            }
            default:
                assert("TODO! UNKNOWN FUNCT3" == 0);
                return;
            }
        }
        case 0b1100111: {
            int64_t imm = (int64_t)((int32_t)instruction)  >> 20;
            uint64_t return_address = emu->cpu.pc + 0x4;
            uint64_t jump_address = ((int64_t)emu->cpu.x_reg[rs1] + (int64_t)imm) & ~1;
            emu->cpu.pc = (jump_address - 0x4);
            emu->cpu.x_reg[rd] = return_address;
            debug_print("DEBUG: jalr %d (x%d)\n",imm,rs1);
            return;
        }
        case 0b1101111: {
            emu->cpu.x_reg[rd] = emu->cpu.pc + 0x4;
            ///
            uint64_t offset = ((int64_t)((int32_t)(instruction & 0x80000000)) >> 11)
            | (instruction & 0xff000)
            | ((instruction >> 9) & 0x800)
            | ((instruction >> 20) & 0x7fe); 
            debug_print("DEBUG: jal %s","\n");
            emu->cpu.pc = (emu->cpu.pc + offset) - 0x4;
            return;
        }
        case 0b1110011:{
            switch (funct7)
            {
                case 0x0:{
                    emulate_syscall(emu);
                    break;
                }
                case 0x1:{
                    todo("ebreak");
                    break;
                }
                default:{
                    todo("invalid funct7")
                    break;
                }
            }
            return;
        }

    default:
        printf("Opcode -> 0x%llx\n",opcode);
        assert("TODO! UNKNOWN OPCODE" == 0);
        return;
    }
}


void emulate_syscall(Emulator* emu){
    // TODO ALLOW CALLBACKS?
    uint64_t syscall = emu->cpu.x_reg[17];
    uint64_t arg0 = emu->cpu.x_reg[10];
    uint64_t arg1 = emu->cpu.x_reg[11];
    uint64_t arg2 = emu->cpu.x_reg[12];
    uint64_t arg3 = emu->cpu.x_reg[13];
    uint64_t arg4 = emu->cpu.x_reg[14];
    uint64_t arg5 = emu->cpu.x_reg[15];
    debug_print("ecall -> 0x%x\n",syscall);
    switch (syscall)
    {
        case 0x42:{
            debug_print("syscall -> writev%s","\n");
            uint64_t file_descriptor = arg0;
            // Read IOVEC Pointer
            iovec* iovec_ptr = (iovec*)vm_read_memory(&emu->mmu,arg1);
            int iovcnt = (int)arg2;
            debug_print("buffer address 0x%llx buffer length %d file_descriptor %d\n",iovec_ptr->iov_base,iovec_ptr->iov_len,file_descriptor);
            // Read Memory Buffer
            ssize_t write_count = 0;
            for (int i = 0; i < iovcnt; i++){
                debug_print("iovec ptr -> 0x%llx\n",iovec_ptr);
                debug_print("fd %d iovec ptr 0x%llx, count %d\n",arg0,arg1,arg2);
                debug_print("iovec data-> 0x%llx iovec data sz -> %d\n",iovec_ptr->iov_base,iovec_ptr->iov_len);
                // TODO dont copy memory into another buffer
                void* buffer = vm_copy_memory(&emu->mmu,(uint64_t)(iovec_ptr->iov_base),iovec_ptr->iov_len);
                // Write it to corresponding file descriptor
                if (file_descriptor == STDOUT_FILENO) {
                    write(file_descriptor,buffer,iovec_ptr->iov_len);
                }
                write_count += iovec_ptr->iov_len;
                free(buffer);
                iovec_ptr++;
            }
            emu->cpu.x_reg[10] = write_count;
            return;
        }
        case 0x1d: {
            debug_print("syscall -> ioctl%s","\n");
            emu->cpu.x_reg[10] = 0;
            return;
        }
        case 0x60:{
            debug_print("syscall -> set_tid_address%s","\n");
            emu->cpu.x_reg[10] = (uint64_t)getpid();
            return;
        }
        case 0x49: {
            debug_print("syscall -> ppoll%s","\n");
            emu->cpu.x_reg[10] = 0;
            panic("shouldnt hit");
            return;
        }
        case 0x38: {
            debug_print("openat%s","\n");
            panic("shouldnt hit");
            return;
        }
        case 0x5e: {
            debug_print("exit syscall%s","\n");
            exit(arg0);
            return;
        }
        default: {
            todo("unimplemeted syscall");
            return;
        }
    }






}
