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
  CrashMap* crash_map_data = (CrashMap*)calloc(1,sizeof(CrashMap));
  Stats* stats_data = (Stats*)calloc(1,sizeof(Stats));
  Corpus* corpus_data = new_corpus("./corpus");
  Emulator* emu = new_emulator(coverage_map_data,crash_map_data,stats_data,corpus_data);
  load_code_segments_into_virtual_memory(emu,code_segment);
  emu->cpu.pc = code_segment->entry_point;
  emu->cpu.stack_pointer = init_stack_virtual_memory(emu,argc,argv); 
  emu->cpu.x_reg[2] = emu->cpu.stack_pointer;
  for (int i = 0; i < code_segment->count; i++){
    free(code_segment->segs[i]->raw_data);
    free(code_segment->segs[i]);
  }
  free(code_segment->segs);
  free(code_segment);
  vm_print(&emu->mmu);
  bool debug = false;
  bool snapshot_taken = false;
  uint64_t snapshot_addr = 0x10236;
  uint64_t restore_addr = 0x10252;
  Emulator* snapshot_immut = NULL;
  for (;;) {
    // Take Snapshot at desired state
    if (!snapshot_taken && emu->cpu.pc == snapshot_addr){
        snapshot_immut = snapshot_vm(emu);
        snapshot_taken = true;
        // TODO Get Address Of Memory To Replace With FuzzCase
    }
    if (emu->cpu.pc == restore_addr){
      /*
      if (emu->coverage->unique_branches_taken > emu->coverage->previous_unique_branches_taken){
        todo("add case to corpus");
        emu->coverage->previous_unique_branches_taken = emu->coverage->unique_branches_taken;
      }
      */
      // Restore VM
      free_emulator(emu);
      emu = snapshot_vm(snapshot_immut);
      // Restore Done
      //todo("choose item from corpus");
      //todo("mutate corpus item");
      //todo("write fuzz case to location")
      emu->coverage = coverage_map_data;
      emu->crashes = crash_map_data;
      emu->stats = stats_data;
      emu->stats->cases++;
      display_stats(emu->stats);
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
