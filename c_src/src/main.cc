#include "loader.h"
#include "emulator.h"
#include "mutate.h"
#include <cstdlib>
#include <stdio.h>


int main(int argc, char **argv) {
  int seed = 0x123;
  srand(seed);
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
  uint64_t snapshot_addr = 0x10256;
  uint64_t restore_addr = 0x1035a;
  Emulator* emu = new_emulator(coverage_map_data,crash_map_data,stats_data,corpus_data,snapshot_addr,restore_addr);
  emu->current_fuzz_case = NULL;
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
  bool debug = true;
  for(;;){
    if (debug){
      getchar();
    }
    uint32_t instruction = fetch(emu);
    execute_instruction(emu,(uint64_t)instruction, generic_record_coverage);
    print_registers(emu);
  }
  free_emulator(emu);
  // TODO's
  // Snapshot and restore efficiency only restore poisoned memory
  // Implement Address Sanitizer 
  return 0;
  /*
  bool snapshot_taken = false;
  Emulator* snapshot_immut = NULL;
  // TODO ask for buffer size somewhere in args maybe
  const size_t payload_size = strlen("Hello From Coffepot!\n");
  FuzzCase fcase_mut = {};
  fcase_mut.size = payload_size;
  fcase_mut.data = (uint8_t*)calloc(payload_size,sizeof(uint8_t));
  */
  // Run until take snapshot
/*
take_snapshot:
  for (;;) {
    if (!snapshot_taken && emu->cpu.pc == snapshot_addr){
        snapshot_immut = snapshot_vm(emu);
        snapshot_taken = true;
        goto fuzz_loop;
    }
    //print_registers(emu);
    uint32_t instruction = fetch(emu);
    execute_instruction(emu,(uint64_t)instruction, generic_record_coverage);
  }
  */
// Restore Loop Here
//fuzz_loop:
//emu->stats->start_time = std::time(0);
  /*
for (;;){
  int corpus_index = rand() % (emu->corpus->count - 1);
  FuzzCase* fcase = &emu->corpus->cases[corpus_index];
  //MutateBuffer(fcase,&fcase_mut);
  //printf("FuzzCase  %d %s\n",corpus_index,(char*)stack_fcase.data);
  //vm_write_buffer(&emu->mmu, 0x113f0, fcase_mut.data, sizeof(uint8_t) * payload_size);
  if (emu->cpu.pc == restore_addr){
    // Check if got more coverage
    if (emu->coverage->unique_branches_taken > emu->coverage->previous_unique_branches_taken){
      emu->coverage->previous_unique_branches_taken = emu->coverage->unique_branches_taken;
      //todo("fix add to corpus");
      //todo("create binary that crashes when certain string is seen");
      //add_to_corpus(emu->corpus,emu->current_fuzz_case);
    }
    // Restore VM 
    free_emulator(emu);
    // TODO Instead of freeing. just copy memory segments that have been poisoned
    emu = snapshot_vm(snapshot_immut);
    emu->corpus = corpus_data;
    emu->coverage = coverage_map_data;
    emu->crashes = crash_map_data;
    emu->stats = stats_data;
    emu->stats->cases++;
    emu->stats->unique_branches = emu->coverage->unique_branches_taken;
    //display_stats(emu->stats);
    if (emu->stats->cases > 10000){
      break;
    }
  } 
}
  */

}
