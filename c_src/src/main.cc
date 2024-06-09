#include "loader.h"
#include "emulator.h"
#include <cstdlib>
#include <stdio.h>


int main(int argc, char **argv) {
  const char *binary_path = *++argv;
  printf("Coffeepot Emulating %s\n", binary_path);
  // Read Binary Into Memory
  FILE *binary_ptr = NULL;
  long binary_size = 0;
  size_t nread = 0;
  char *binary_buffer = NULL;
  //
  binary_ptr = fopen(binary_path, "rb");
  if (binary_ptr == NULL) {
    fprintf(stderr, "ERROR: Failed to read file %s", binary_path);
    return -1;
  }
  fseek(binary_ptr, 0, SEEK_END);
  binary_size = ftell(binary_ptr);
  rewind(binary_ptr);
  binary_buffer = (char *)calloc(1, binary_size);
  nread = fread(binary_buffer, 1, binary_size, binary_ptr);
  // Parse Elf Headers
  CodeSegments* code_segment = parse_elf_segments(binary_buffer,nread);
  free(binary_buffer);
  fclose(binary_ptr);
  // Create Coverage Map Memory
  CoverageMap* coverage_map_data = (CoverageMap*)calloc(1,sizeof(CoverageMap));
  // Create Virtual Memory
  Emulator* emu = new_emulator(coverage_map_data);
  load_code_segments_into_virtual_memory(emu,code_segment);
  //printf("Code Loaded At 0x%x\n",code_segment->base_address);
  // INITALIZE CPU REGISTERS
  emu->cpu.pc = code_segment->entry_point;
  emu->cpu.stack_pointer = init_stack_virtual_memory(emu,argc,argv); 
  emu->cpu.x_reg[2] = emu->cpu.stack_pointer;
  // Free elf segments
  for (int i = 0; i < code_segment->count; i++){
    free(code_segment->segs[i]->raw_data);
    free(code_segment->segs[i]);
  }
  free(code_segment->segs);
  free(code_segment);
  // PRINT SEGMENTS
  vm_print(&emu->mmu);
  /// Emulator Basic Loop
  int t = 0;
  bool debug = false;
  bool snapshot_taken = false;
  uint64_t snapshot_addr = 0x10236;
  uint64_t restore_addr = 0x10252;
  Emulator* snapshot_immut = NULL;
  uint64_t iterations = 0;
  for (;;) {
    // Take Snapshot at desired state
    if (!snapshot_taken && emu->cpu.pc == snapshot_addr){
        snapshot_immut = snapshot_vm(emu);
        snapshot_taken = true;
    }
    // Restore vm back to snapshot
    if (emu->cpu.pc == restore_addr){
      // TODO Check if our coverage increased since last case
      // If it did add case to corpus
      // free emulator memory then clone the immutable snapshot and assign that to emulator
      free_emulator(emu);
      emu = snapshot_vm(snapshot_immut);
      // keep coverage map alive
      emu->coverage = coverage_map_data;
      // keep crash map alive
      // keep statistics alive
      iterations++;
      printf("%lld iterations\n",iterations);
    }
    // Execute Instructions
    print_registers(emu);
    uint32_t instruction = fetch(emu);
    execute_instruction(emu,(uint64_t)instruction, generic_record_coverage);
  }

  free_emulator(emu);
  // TODO....
  // Set Memory Permissions Function
  // Crash Gathering Callback
  // Coverage Gathering Callback
  // Enable Memory Swapping For FuzzCases
  // Better Memory Permisssions
  return 0;
}
