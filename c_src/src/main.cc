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
  printf("READ %lu bytes\n", nread);
  // Parse Elf Headers
  CodeSegment* code_segment = parse_elf_segments(binary_buffer,nread);
  free(binary_buffer);
  fclose(binary_ptr);
  // Create Virtual Memory
  Emulator* emu = new_emulator();
  load_code_segments_into_virtual_memory(emu,code_segment);
  printf("Code Loaded At 0x%x\n",code_segment->base_address);
  free(code_segment->raw_data);
  free(code_segment);
  vm_print(&emu->mmu);
  //
  free_emulator(emu);

  //.....
  // Copy Segments Into Memory
  // Set Memory Permissions
  // Set EntryPoint
  // Fetch
  // Decode
  // Execute
  // Snapshot
  // Restore
  // Enable Memory Swapping For FuzzCases
  return 0;
}
