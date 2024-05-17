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
        debug_print("[%d] DEBUG SEGMENT: 0x%x-0x%x size 0x%0x perms 0x%x\n",i,mmu->virtual_memory[i].range.start,mmu->virtual_memory[i].range.end,mmu->virtual_memory[i].data_size,mmu->virtual_memory[i].perms);
    }
}

Segment* vm_get_segment(MMU* mmu, uint64_t address){
    //fprintf(stderr,"DEBUG: GETTING SEGMENT 0x%x\n",address);
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
        //fprintf(stderr,"DEBUG: ALLOC AT 0x%x\n",base);
        if (mmu->segment_count + 1 > mmu->segment_capacity){
            // realloc
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

void load_code_segments_into_virtual_memory(Emulator* emu ,CodeSegment* code){
  uint64_t text_section_base = vm_alloc(&emu->mmu, code->base_address,code->total_size, READ|WRITE|EXEC);
  vm_copy(&emu->mmu,code->raw_data,code->total_size,code->base_address);
}

uint64_t init_stack_virtual_memory(Emulator* emu, int argc, char** argv){
  uint64_t stack_base = vm_alloc(&emu->mmu, 0, 1024*1024, READ | WRITE);
  uint64_t stack_pointer = stack_base + (1024*1024);
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0x0000000);
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0x0000000);
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 0x0000000);
  /// loop over args and write them
  // heap
  uint64_t string_address = vm_alloc(&emu->mmu,0,1024,READ|WRITE);
  // for now we just right AAAAA
  vm_write_double_word(&emu->mmu,string_address,0x41414141);
  // write heap pointer to stack
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, string_address);
  // Write Argc
  stack_pointer -= 0x8;
  vm_write_double_word(&emu->mmu,stack_pointer, 1);
  // Return SP
  return stack_pointer;
}


void vm_write_double_word(MMU* mmu, uint64_t address, uint64_t value)  {
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    //printf("Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    s->data[index] = (value & 0xff);
    s->data[index + 1] = ((value >> 8 ) & 0xff);
    s->data[index + 2] = ((value >> 16 ) & 0xff);
    s->data[index + 3] = ((value >> 24 ) & 0xff);
    s->data[index + 4] = ((value >> 32 ) & 0xff);
    s->data[index + 5] = ((value >> 40 ) & 0xff);
    s->data[index + 6] = ((value >> 48 ) & 0xff);
    s->data[index + 7] = ((value >> 56 ) & 0xff);
}


uint64_t vm_read_word(MMU* mmu, uint64_t address){
    Segment* s = vm_get_segment(mmu, address);
    if (s == NULL){
        assert("TODO HANDLE SEGFAULT! WITH A CALLBACK" == 0);
    }
    uint64_t index = address - s->range.start;
    //fprintf(stderr,"DEBUG: Address 0x%x memory base 0x%x segment offset 0x%x\n",address, s->range.start,index);
    return (uint64_t)(s->data[index])
        | ((uint64_t)(s->data[index + 1]) << 8)
        | ((uint64_t)(s->data[index + 2]) << 16)
        | ((uint64_t)(s->data[index + 3]) << 24);
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
    debug_print("pc: 0x%x\n",emu->cpu.pc);
    for (int i = 0; i < 32; i++){
        debug_print("x%d 0x%x\n",i,emu->cpu.x_reg[i]);
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
            case 0x2: {
                uint64_t rd = ((instruction >> 2) & 0x7) + 8;
                uint64_t rs1 = ((instruction >> 7) & 0x7) + 8;
                uint64_t offset = ((instruction << 1) & 0x40) // imm[6]
                            | ((instruction >> 7) & 0x38) // imm[5:3]
                            | ((instruction >> 4) & 0x4); // imm[2]
                debug_print("DEBUG c_lw x%d, 0x%x (x%d)\n",rd,offset,rs1);
                uint64_t memory_address = emu->cpu.x_reg[rs1] + offset;
                uint64_t result = vm_read_word(&emu->mmu,memory_address);
                emu->cpu.x_reg[rd] = (uint64_t)((int64_t)((int32_t)result));
                break;
            }
            default: {
                assert("UNKNOWN FUNC3 QUADRANT 0");
                }
            }
        break;
    }
    case 0b01: {
        debug_print("DEBUG QUADRANT %d\n",1);
        break;
    }
    case 0b10: {
        debug_print("DEBUG QUADRANT %d\n",2);
        switch (funct3) {
            case 0x1: {
                break;
            }
            case 0x2: {
                break;
            }
            case 0x3: {
                break;
            }
            case 0x4: {
                uint64_t left = (instruction >> 12) & 0x1;
                uint64_t right = (instruction >> 2) & 0x1f;
                if (left == 0 && right == 0){
                    debug_print("DEBUG c_jr %s\n","quadrant 2");
                    break;
                } else if (left == 0 && right != 0){
                    debug_print("DEBUG c_mv %s\n","quadrant 2");
                    uint64_t rd = (instruction >> 7 ) & 0x1f;
                    uint64_t rs2 = (instruction >> 2 ) & 0x1f;
                    if (rs2 != 0){
                        emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs2];
                    }
                    debug_print("DEBUG a0 => 0x%x sp => 0x%x\n",emu->cpu.x_reg[rd],emu->cpu.x_reg[2]);
                    break;
                } else if (left == 1 && right == 0){
                    debug_print("DEBUG c_ebreak %s\n","quadrant 2");
                    break;
                } else if (left == 1 && right != 0){
                    debug_print("DEBUG c_add %s\n","quadrant 2");
                    break;
                } else {
                    assert("QUADRANT 2 INVALID INSTRUCTION" != 0 );
                }
                break;
            }
            default:
                assert("INVALID FUNCT3 QUADRANT 2" != 0);
        }
        break;
    }
    default:
        assert("INVALID QUADRANT " != 0);
    }
}





static void execute(Emulator* emu, uint64_t instruction){
    // decode get what we need
    uint64_t opcode = instruction & 0x0000007f;
    uint64_t rd  = (instruction & 0x00000f80) >> 7;
    uint64_t rs1 = (instruction & 0x000f8000) >> 15;
    uint64_t rs2 = (instruction & 0x01f00000) >> 20;
    uint64_t funct3 = (instruction & 0x00007000) >> 12;
    switch (opcode)
    {
        case 0b0010111: {
            uint64_t imm = (uint64_t)((int64_t)((int32_t)(instruction & 0xfffff000)));
            emu->cpu.x_reg[rd] = emu->cpu.pc + imm;
            debug_print("DEBUG: auipc x%d,0x%01x\n",rd,imm);
            break;
        }
        case 0b00010011: {
            uint64_t imm = (uint64_t)((( (int64_t)((int32_t)instruction)) >> 20));
            switch (funct3)
            {
            case 0x0:{
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] + imm;
                debug_print("DEBUG: addi x%d,x%d, 0x%x\n",rd,rs1,imm);
                break;
            }
            case 0x7: {
                emu->cpu.x_reg[rd] = emu->cpu.x_reg[rs1] & imm;
                debug_print("DEBUG: andi x%d,x%d, 0x%x\n",rd,rs1,imm);
                break;
            }
            default:
                assert("TODO! UNKNOWN FUNCT3");
            }
            break;
        }
        case 0b1100111: {
            int64_t imm = (int64_t)((int32_t)instruction)  >> 20;
            uint64_t return_address = emu->cpu.pc + 0x4;
            uint64_t jump_address = ((int64_t)emu->cpu.x_reg[rs1] + (int64_t)imm) & ~1;
            emu->cpu.pc = (jump_address - 0x4);
            emu->cpu.x_reg[rd] = return_address;
            debug_print("DEBUG: jalr %d (x%d)\n",imm,rs1);
            break;
        }
    default:
        assert("TODO! UNKNOWN OPCODE" == 0);
    }
}

///             0b0010111 => self.auipc(rd, (instruction & 0xfffff000) as i32 as i64 as u64),
