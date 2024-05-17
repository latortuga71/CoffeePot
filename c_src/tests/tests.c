#include "loader.h"
#include "emulator.h"
#include <cstdlib>
#include <stdio.h>

int stack_init_test(Emulator* emu, int argc, char** argv){
    return 1;
}

int jalr_test(Emulator* emu,uint64_t instruction){
    emu->cpu.pc = 0x10164;
    emu->cpu.x_reg[6] = 0x10160;
    uint64_t expected = 0x10168;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.pc;
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","jalr_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[-] TEST PASSED: %s \n","jalr_test");
    return 1;
}

int c_mv_test(Emulator* emu,uint64_t instruction){
    uint64_t expected = 0x1234;
    emu->cpu.x_reg[2] = 0x1234;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[10];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_mv_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[-] TEST PASSED: %s \n","c_mv_test");
    return 1;
}

int andi_test(Emulator* emu,uint64_t instruction){
    emu->cpu.x_reg[2] = 0x40007ffa40;
    uint64_t expected = emu->cpu.x_reg[2];
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[2];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","andi_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[-] TEST PASSED: %s \n","andi_test");
    return 1;
}

int addi_test(Emulator* emu,uint64_t instruction){
    uint64_t expected = 0x12d58;
    emu->cpu.x_reg[3] = 0x1314a;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[3];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","addi_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[-] TEST PASSED: %s \n","addi_test");
    return 1;
}

int auipc_test(Emulator* emu,uint64_t instruction){
    uint64_t expected = 0x1314a;
    emu->cpu.pc = 0x1014a;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[3];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s \n","auipc_test");
      return 0;
    }
      fprintf(stderr,"[-] TEST PASSED: %s \n","auipc_test");
    return 1;
}

int load_elf_test(){
  FILE *binary_ptr = NULL;
  long binary_size = 0;
  size_t nread = 0;
  char *binary_buffer = NULL;
  //
  binary_ptr = fopen("./tests/hello_test.bin", "rb");
  if (binary_ptr == NULL) {
    fprintf(stderr, "ERROR: Failed to read file\n");
    return 0;
  }
  fseek(binary_ptr, 0, SEEK_END);
  binary_size = ftell(binary_ptr);
  rewind(binary_ptr);
  binary_buffer = (char *)calloc(1, binary_size);
  nread = fread(binary_buffer, 1, binary_size, binary_ptr);
  // Parse Elf Headers
  CodeSegment* code_segment = parse_elf_segments(binary_buffer,nread);
  if (code_segment->entry_point != 0x1014A){
    free(binary_buffer);
    fclose(binary_ptr);
    fprintf(stderr,"[-] TEST FAILED: %s \n","load_elf_test");
    return 0;
  }
  //load_code_segments_into_virtual_memory(emu,code_segment);
  free(binary_buffer);
  fclose(binary_ptr);
    fprintf(stderr,"[-] TEST PASSED: %s \n","load_elf_test");
  return 1;
}



int main(){
  Emulator* emu = new_emulator();
  int passed_tests = 0;
  int total_tests = 6;
  if (load_elf_test())
    passed_tests +=1;
  if (auipc_test(emu, 0x00003197))
    passed_tests +=1;
  if (addi_test(emu, 0xc0e18193))
    passed_tests +=1;
  if (c_mv_test(emu, 0x850a))
    passed_tests +=1;
  if (andi_test(emu, 0xff017113))
    passed_tests +=1;
  if (jalr_test(emu, 0x00830067))
    passed_tests +=1;
  fprintf(stderr,"[+] %d/%d TESTS PASSED\n",passed_tests,total_tests);
  return 0;
}
