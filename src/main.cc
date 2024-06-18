#include "loader.h"
#include "emulator.h"
#include "mutate.h"
#include <cstdlib>
#include <stdio.h>


CodeSegments* parse_elf(const char* path){
  // Read Binary Into Memory
  FILE *binary_ptr = NULL;
  long binary_size = 0;
  size_t nread = 0;
  char *binary_buffer = NULL;
  //
  binary_ptr = fopen(path, "rb");
  if (binary_ptr == NULL) {
    fprintf(stderr, "ERROR: Failed to read file %s", path);
    assert("Failed to read elf" == 0);
  }
  fseek(binary_ptr, 0, SEEK_END);
  binary_size = ftell(binary_ptr);
  rewind(binary_ptr);
  binary_buffer = (char *)calloc(1, binary_size);
  nread = fread(binary_buffer, 1, binary_size, binary_ptr);
  CodeSegments* code_segment = parse_elf_segments(binary_buffer,nread);
  free(binary_buffer);
  fclose(binary_ptr);
  return code_segment;
}


void delete_code_segments(CodeSegments* code_segment){
  for (int i = 0; i < code_segment->count; i++){
    free(code_segment->segs[i]->raw_data);
    free(code_segment->segs[i]);
  }
  free(code_segment->segs);
  free(code_segment);
}

int debug_main_no_snapshot(int argc, char **argv) {
  int seed = 0x123;
  srand(seed);
  const char *binary_path = *++argv;
  printf("Coffeepot Emulating %s\n", binary_path);
  // Read Elf Segments Into Memory
  CodeSegments* code_segment =  parse_elf(binary_path);
  //
  CoverageMap* coverage_map_data = (CoverageMap*)calloc(1,sizeof(CoverageMap));
  // Coverage Configv
  CrashMap* crash_map_data = (CrashMap*)calloc(1,sizeof(CrashMap));
  // Stats Config
  Stats* stats_data = (Stats*)calloc(1,sizeof(Stats));
  // Corpus Config
  Corpus* corpus_data = new_corpus("./corpus");
  // SnapShot & Restore Config
  uint64_t snapshot_addr = 0x0;
  uint64_t restore_addr = 0x0;
  // Create Emulator
  Emulator* emu = new_emulator(coverage_map_data,crash_map_data,stats_data,corpus_data,snapshot_addr,restore_addr);
  emu->current_fuzz_case = NULL;
  emu->cpu.pc = code_segment->entry_point;
  load_code_segments_into_virtual_memory(emu,code_segment);
  init_stack_virtual_memory(emu,argc,argv,generic_record_crashes); 
  delete_code_segments(code_segment);
  bool debug = false;
  for(;;){
    if (emu->cpu.pc == 0x11eea){
      debug = false;
    }
    if (debug)
      getchar();
    uint32_t instruction = fetch(emu,generic_record_crashes);
    execute_instruction(emu,(uint64_t)instruction, generic_record_coverage,generic_record_crashes);
    print_registers(emu);
  }
  free_emulator(emu);
  return 0;
}

int main(int argc, char **argv) {
  debug_main_no_snapshot(argc,argv);
  return 0;
  int seed = 0x1234;
  srand(seed);
  const char *binary_path = *++argv;
  printf("Coffeepot Emulating %s\n", binary_path);
  // Read Elf Segments Into Memory
  CodeSegments* code_segment =  parse_elf(binary_path);
  //
  CoverageMap* coverage_map_data = (CoverageMap*)calloc(1,sizeof(CoverageMap));
  // Coverage Configv
  CrashMap* crash_map_data = (CrashMap*)calloc(1,sizeof(CrashMap));
  // Stats Config
  Stats* stats_data = (Stats*)calloc(1,sizeof(Stats));
  // Corpus Config
  Corpus* corpus_data = new_corpus("./corpus");
  // SnapShot & Restore Config
  uint64_t snapshot_addr = 0x10256;
  uint64_t restore_addr = 0x1035a;
  // Create Emulator
  Emulator* emu = new_emulator(coverage_map_data,crash_map_data,stats_data,corpus_data,snapshot_addr,restore_addr);
  emu->current_fuzz_case = NULL;
  emu->cpu.pc = code_segment->entry_point;
  load_code_segments_into_virtual_memory(emu,code_segment);
  init_stack_virtual_memory(emu,argc,argv,generic_record_crashes); 
  delete_code_segments(code_segment);
  bool debug = false;
  bool snapshot_taken = false;
  Emulator* snapshot_immut = NULL;
  do {
    uint32_t instruction = fetch(emu,generic_record_crashes);
    execute_instruction(emu,(uint64_t)instruction, generic_record_coverage,generic_record_crashes);
  } while(emu->cpu.pc != emu->snapshot_address);
  if (emu->coverage->unique_branches_taken > emu->coverage->previous_unique_branches_taken){
    emu->coverage->previous_unique_branches_taken = emu->coverage->unique_branches_taken;
  }
  snapshot_immut = snapshot_vm(emu);
  printf("Snapshot taken!\n");
  printf("Fuzz Loop Begins Here\n");
  // Setup fuzzcase that gets mutated so we dont alloc a bunch of times
  FuzzCase fcase_mut = {0};
  fcase_mut.size = strlen("Hello From CoffeePot!\n");
  fcase_mut.data = (uint8_t*)calloc(fcase_mut.size,sizeof(uint8_t));
  emu->stats->start_time = std::time(0);
  for (;;){
    // Here We Mutate The Buffer
    int corpus_index = rand() % ((emu->corpus->count) - 0);
    FuzzCase* fcase = &emu->corpus->cases[corpus_index];
    MutateBuffer(fcase,&fcase_mut);
    vm_write_buffer(&emu->mmu, 0x113f0, fcase_mut.data, sizeof(uint8_t) * fcase_mut.size);
    emu->current_fuzz_case = &fcase_mut;
    // Execute Normally
    do {
      uint32_t instruction = fetch(emu,generic_record_crashes);
      execute_instruction(emu,(uint64_t)instruction, generic_record_coverage,generic_record_crashes);
      if (emu->crashed){
        emu->stats->crashes++;
        break;
      }
    } while( emu->cpu.pc != restore_addr);
    // If we got more coverage add it to the corpus
    if (emu->coverage->unique_branches_taken > emu->coverage->previous_unique_branches_taken){
      emu->coverage->previous_unique_branches_taken = emu->coverage->unique_branches_taken;
      add_to_corpus(emu->corpus, &fcase_mut);
    }
    restore_vm(emu,snapshot_immut);
    emu->corpus = corpus_data;
    emu->coverage = coverage_map_data;
    emu->crashes = crash_map_data;
    emu->stats = stats_data;
    emu->stats->cases++;
    emu->stats->unique_branches = emu->coverage->unique_branches_taken;
    if (((int)emu->stats->cases % 10000) == 0)
      display_stats(emu->stats,emu->corpus);
  }
  free_emulator(emu);
  return 0;
}

  // TODO's
  // TODO implement faster way to get segments without hardcoding addresses probably need to use a map or 1 giant array
  // Implement Poisoned memory for each segment on write calls
  // use flag to determine if this should occur since we want it after we snapshot so only at that point we do the dirty mem
  // above can also be used to check memory permissions
  // Implement Address Sanitizer 
  // Add more complex binaries (aka complete instruction set,syscalls etc)