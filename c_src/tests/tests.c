#include "loader.h"
#include "emulator.h"
#include <cstdlib>
#include <stdio.h>

void reset_color(){
  fprintf(stderr,"\033[0m");
}

int stack_init_test(Emulator* emu, int argc, char** argv){
    return 1;
}

int c_addi16sp_test(Emulator* emu,uint64_t instruction){
    emu->cpu.x_reg[2] = 0x40007ffa20;
    uint64_t expected = 0x40007ff8a0;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[2];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_addi16sp_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_addi16sp_test");
    return 1;
}

int c_slli_test(Emulator* emu,uint64_t instruction){
    emu->cpu.x_reg[15] = 0x2;
    uint64_t expected = 0x10;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[15];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_slli_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_slli_test");
    return 1;
}

int c_ld_test(Emulator* emu,uint64_t instruction,uint64_t stack_pointer){
    emu->cpu.x_reg[11] = 0x0;
    uint64_t expected = 0x42424242;
    emu->cpu.x_reg[12] = stack_pointer + 0x8;
    uint64_t address = emu->cpu.x_reg[12];
    vm_write_double_word(&emu->mmu,address,0x42424242);
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[11];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_ld_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_ld_test");
    return 1;
}

int c_sdsp_test(Emulator* emu,uint64_t instruction,uint64_t stack_pointer){
    emu->cpu.x_reg[9] = 0x41414141;
    uint64_t expected = 0x41414141;
    emu->cpu.x_reg[2] = stack_pointer;
    uint64_t address = emu->cpu.x_reg[2] + 0x8;
    execute_instruction(emu,instruction);
    uint64_t result = vm_read_double_word(&emu->mmu,address);
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_sdsp_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_sdsp_test");
    return 1;
}

int c_li_test(Emulator* emu, uint64_t instruction){
    emu->cpu.x_reg[15] = 0x0;
    uint64_t expected = 0x0;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[15];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_li_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_li_test");
    return 1;
}

int c_addi_test(Emulator* emu ,uint64_t instruction){
    emu->cpu.x_reg[2] = 0x40007ffa40;
    uint64_t expected = 0x40007ffa20;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[2];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_addi_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_addi_test");
    return 1;
}

int lui_test(Emulator* emu,uint64_t instruction){
    emu->cpu.x_reg[10] = 0;
    uint64_t expected = 0x10000;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[10];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","lui_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","lui_test");
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
      fprintf(stderr,"[+] TEST PASSED: %s \n","jalr_test");
    return 1;
}

int c_lw_test(Emulator* emu,uint64_t instruction){
    uint64_t expected = 0x1;
    emu->cpu.x_reg[10] = 0x112cd0;
    emu->cpu.x_reg[11] = 0x0;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[11];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","c_lw_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_lw_test");
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
      fprintf(stderr,"[+] TEST PASSED: %s \n","c_mv_test");
    return 1;
}

int add_test(Emulator* emu,uint64_t instruction){
    emu->cpu.x_reg[12] = 0x40007ffa48;
    emu->cpu.x_reg[15] = 0x10;
    emu->cpu.x_reg[10] = 0x0;
    uint64_t expected = 0x40007ffa58;
    execute_instruction(emu,instruction);
    uint64_t result = emu->cpu.x_reg[10];
    if (expected != result){
      fprintf(stderr,"[-] TEST FAILED: %s expected 0x%x result 0x%x\n","add_test",expected,result);
      return 0;
    }
      fprintf(stderr,"[+] TEST PASSED: %s \n","add_test");
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
      fprintf(stderr,"[+] TEST PASSED: %s \n","andi_test");
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
      fprintf(stderr,"[+] TEST PASSED: %s \n","addi_test");
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
      fprintf(stderr,"[+] TEST PASSED: %s \n","auipc_test");
    return 1;
}

int load_elf_test(Emulator* emu){
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
  load_code_segments_into_virtual_memory(emu,code_segment);
  free(binary_buffer);
  fclose(binary_ptr);
    fprintf(stderr,"[+] TEST PASSED: %s \n","load_elf_test");
  return 1;
}



int main(){
  Emulator* emu = new_emulator();
  int passed_tests = 0;
  int total_tests = 15;
  if (load_elf_test(emu))
    passed_tests +=1;
  emu->cpu.stack_pointer = init_stack_virtual_memory(emu,1,NULL); 
  uint64_t stack_pointer_og = emu->cpu.stack_pointer;
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
  if (c_lw_test(emu, 0x410c))
    passed_tests +=1;
  if (lui_test(emu, 0x00010537))
    passed_tests +=1;
  if (c_li_test(emu, 0x4781))
    passed_tests +=1;
  if (c_addi_test(emu, 0x1101))
    passed_tests +=1;
  if (c_sdsp_test(emu, 0xe426,stack_pointer_og))
    passed_tests +=1;
  if (c_ld_test(emu, 0x620c,stack_pointer_og))
    passed_tests +=1;
  if (c_slli_test(emu, 0x078e))
    passed_tests +=1;
  if (add_test(emu,0x00f60533))
    passed_tests +=1;
  if (c_addi16sp_test(emu,0x7109))
    passed_tests +=1;

  ///////
  if (passed_tests == total_tests){
    fprintf(stderr,"\033[0;32m[+] %d/%d ALL TESTS PASSED\n",passed_tests,total_tests);
  } else {
    fprintf(stderr,"\033[1;31mFAIL [-] %d/%d TESTS PASSED\n",passed_tests,total_tests);
  }
  reset_color();
  return 0;
}
