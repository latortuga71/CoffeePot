#include "emulator.h"



Emulator* new_emulator(){
    Emulator* emu = (Emulator*)calloc(1,sizeof(Emulator));
    emu->mmu.next_allocation_base = 0;
    emu->mmu.virtual_memory = (Segment*)calloc(100,sizeof(Segment));
    emu->mmu.segment_count = 0;
    emu->mmu.segment_capacity = 100;
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
    debug_print("DEBUG: GETTING SEGMENT 0x%llx\n",address);
    for (int i = 0; i < mmu->segment_count; i++){
        if (address >= mmu->virtual_memory[i].range.start && address < mmu->virtual_memory[i].range.end){
            return &mmu->virtual_memory[i];
        }
    }
    vm_print(mmu);
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
    uint64_t page_size = 4096;
    if (base_address == 0){
        // makre sure base is 8 byte aligned
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

uint64_t init_stack_virtual_memory(Emulator* emu, int argc, char** argv){
  uint64_t stack_base = 0x4000000000;
  uint64_t alloc_base = vm_alloc(&emu->mmu, stack_base, 0x1024*0x1024, READ | WRITE);
  debug_print("base 0x%llx\n",alloc_base);
  uint64_t stack_end = stack_base + (0x1024 * 0x1024);
  uint64_t stack_pointer = stack_end - (1024 * 1024);
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0);

  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0);

  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0);
  /// loop over args and write them
  while (*argv != NULL){
    uint64_t string_address = vm_alloc(&emu->mmu,0 , 1024 ,READ|WRITE);
    vm_write_string(&emu->mmu,string_address,*argv);
    stack_pointer -= 0x8;
    vm_write_double_word(&emu->mmu, stack_pointer, string_address);
    argv++;
  }
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, argc);
  debug_print("0x%llx\n",stack_pointer);
  return stack_pointer;
}


void vm_write_byte(MMU* mmu, uint64_t address, uint64_t value)  {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        panic("TODO HANDLE SEGFAULT! WITH A CALLBACK");
    }
    uint64_t index = address - s->range.start;
    debug_print("WRITE BYTE Writing 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (uint8_t)(value);
}

void vm_write_word(MMU* mmu, uint64_t address, uint64_t value)  {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        panic("TODO HANDLE SEGFAULT! WITH A CALLBACK");
    }
    uint64_t index = address - s->range.start;
    debug_print("Writing WORD 0x%llx to 0x%llx\n",value,address);
    s->data[index] = (value & 0xff);
    s->data[index + 1] = ((value >> 8 ) & 0xff);
    s->data[index + 2] = ((value >> 16 ) & 0xff);
    s->data[index + 3] = ((value >> 24 ) & 0xff);
}


void vm_write_double_word(MMU* mmu, uint64_t address, uint64_t value)  {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        debug_print("Attempted to write too 0x%llx\n",address);
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
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

void* vm_copy_memory(MMU* mmu,uint64_t address,size_t count) {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    uint8_t* copy = (uint8_t*)malloc(sizeof(uint8_t) * count);
    memset(copy,0,count);
    //printf("-> %s\n",(char*)&s->data[index]);
    //printf("-> %s \0\n",copy);
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

uint64_t vm_read_double_word(MMU* mmu, uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
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

uint64_t vm_read_word(MMU* mmu, uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    debug_print("READING WORD 0x%llx\n",address);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8)
        | ((uint64_t)(s->data[index + 2]) << 16)
        | ((uint64_t)(s->data[index + 3]) << 24);
}

uint64_t vm_read_half(MMU* mmu, uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    debug_print("READING HALF 0x%llx\n",address);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8);
}
uint64_t vm_read_byte(MMU* mmu, uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    debug_print("READING WORD 0x%llx\n",address);
    return (uint64_t)(s->data[index]);
}

uint32_t fetch(Emulator* emu) {
    //fprintf(stderr,"DEBUG: FETCHING INSTRUCTION 0x%x\n",emu->cpu.pc);
    Segment* segment = vm_get_segment(&emu->mmu,emu->cpu.pc);
    if (segment == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    return (uint32_t)vm_read_word(&emu->mmu,emu->cpu.pc);
}

void print_registers(Emulator* emu){
    debug_print("pc: 0x%llx\n",emu->cpu.pc);
    for (int i = 0; i < 32; i++){
        debug_print("x%d 0x%llx\n",i,emu->cpu.x_reg[i]);
    }
}

void execute_instruction(Emulator* emu, uint64_t instruction){
    emu->cpu.x_reg[0] = 0;
    if ((0x3 & instruction) != 0x3) {
        //fprintf(stderr,"DEBUG: COMPRESSED\n");
        debug_print("DEBUG: COMPRESSED 0x%02x\n",(uint16_t)instruction);
        execute_compressed(emu, instruction);
        emu->cpu.pc += 0x2;
    } else {
        //fprintf(stderr,"DEBUG: NOT COMPRESSED\n");
        debug_print("DEBUG: 0x%08x\n", instruction);
        execute(emu,instruction);
        emu->cpu.pc += 0x4;
    }
}

static void execute_compressed(Emulator* emu, uint64_t instruction){
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
                uint64_t result = vm_read_word(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)result));
                return;
            }
            case 0x3: {
                uint64_t rd = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0xc0) // imm[7:6]
                            | ((instruction >> 7) & 0x38); // imm[5:3]
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                uint64_t result = vm_read_double_word(&emu->mmu,memory_address);
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
                vm_write_word(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
                debug_print("c.sw x%d,%d(x%d)\n",rs2,offset,rs1);
                return;
            }
            case 0x7: {
                uint64_t rs2 = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0xc0) 
                | ((instruction >> 7) & 0x38);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                vm_write_double_word(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
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
                    emu->cpu.pc = (emu->cpu.pc + offset) - 0x2;
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
                    emu->cpu.pc = (emu->cpu.pc + offset) - 0x2;
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
                todo("c.lwsp");
                return;
            }
            case 0x3: {
                uint64_t rd = (instruction >> 7 ) & 0x1f;
                uint64_t offset = ((instruction << 4) & 0x1c0) |
                ((instruction >> 7) & 0x20) |
                ((instruction >> 2) & 0x18);
                uint64_t memory_address = emu->cpu.x_reg[2] + offset;
                debug_print("c.ldsp%s","\n");
                uint64_t result = vm_read_double_word(&emu->mmu,memory_address);
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
                vm_write_word(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
                return;
            }
            case 0x7: {
                uint64_t rs2 = (instruction >> 2) & 0x1f;
                uint64_t offset = ((instruction >> 1 ) & 0x1c0) | ((instruction >> 7) & 0x38);
                uint64_t memory_address = emu->cpu.x_reg[2] + offset;
                debug_print("c.sdsp x%x,0x%x(sp)\n",rs2,offset);
                vm_write_double_word(&emu->mmu, memory_address, emu->cpu.x_reg[rs2]);
                uint64_t after = vm_read_double_word(&emu->mmu, memory_address);
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





static void execute(Emulator* emu, uint64_t instruction){
    // decode get what we need
    uint64_t opcode = instruction & 0x0000007f;
    uint64_t rd  = (instruction & 0x00000f80) >> 7;
    uint64_t rs1 = (instruction & 0x000f8000) >> 15;
    uint64_t rs2 = (instruction & 0x01f00000) >> 20;
    uint64_t funct3 = (instruction & 0x00007000) >> 12;
    uint64_t funct7 = (instruction & 0xfe000000) >> 25;
    switch (opcode)
    {
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
                uint64_t value = vm_read_half(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = (int64_t)((int16_t)(value));
                return;
            }
            case 0x2: {
                debug_print("lw%s","\n");
                uint64_t value = vm_read_word(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x3: {
                debug_print("ld%s","\n");
                uint64_t value = vm_read_double_word(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x4: {
                debug_print("lbu%s","\n");
                uint64_t value = vm_read_byte(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = value;
                return;
            }
            case 0x5: {
                debug_print("lhu%s","\n");
                uint64_t value = vm_read_half(&emu->mmu,memory_address);
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
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                }
                return;
            }
            case 0x1: {
                debug_print("bne x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] != emu->cpu.x_reg[rs2]){
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                }
                return;
            }
            case 0x4: {
                debug_print("blt x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if ((int64_t)(emu->cpu.x_reg[rs1]) < (int64_t)(emu->cpu.x_reg[rs2])){
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
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
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                }
                return;
            }
            case 0x6: {
                debug_print("bltu x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] < emu->cpu.x_reg[rs2]){
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
                }
                return;
            }
            case 0x7: {
                debug_print("bgeu x%d, x%d, 0x%x\n",rs1,rs2,emu->cpu.pc + imm);
                if (emu->cpu.x_reg[rs1] >= emu->cpu.x_reg[rs2]){
                    emu->cpu.pc = (emu->cpu.pc + imm) - 0x4;
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
                vm_write_byte(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
                return;
            }
            case 0x1: {
                todo("sh");
                return;
            }
            case 0x2: {
                debug_print("sw x%d,0x%x,x%d\n",rs2,offset,rs1);
                vm_write_word(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
                return;
            }
            case 0x3: {
                debug_print("sd x%d,%d,x%d\n",rs2,offset,rs1);
                vm_write_double_word(&emu->mmu,memory_address,emu->cpu.x_reg[rs2]);
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
                if (iovec_ptr->iov_len < 1){
                    continue;
                }
                void* buffer = vm_copy_memory(&emu->mmu,(uint64_t)(iovec_ptr->iov_base),iovec_ptr->iov_len);
                // Write it to corresponding file descriptor
                if (file_descriptor == STDOUT_FILENO) {
                    //printf("%s",(char*)buffer);
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
            uint64_t value = vm_read_double_word(&emu->mmu,arg1);
            printf("-> %s\n",vm_read_string(&emu->mmu,arg1));
            debug_print("dirfd: %d path: 0x%llx\n",arg0,arg1);
            debug_print("path dereferenced 0x%llx\n",value);
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