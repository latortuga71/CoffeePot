#include "loader.h"
#include <assert.h>
#include <stdio.h>

CodeSegments* parse_elf_segments(char *elf_data,size_t file_size) {
  ElfFileHdr *file_header = (ElfFileHdr *)elf_data;
  //printf("ELF Header -> %s\n", file_header->e_ident);
  //printf("ELF Entry -> 0x%x\n", file_header->e_entry);
  //printf("ELF LOAD SEGMENTS -> %d\n", file_header->e_phnum);
  uint64_t ph_offset = file_header->e_phoff;
  uint64_t ph_size = file_header->e_phentsize;
  int offset = 0;
  ElfProgHdr* program_header_entry = NULL;
  // Prepare buffer that will contain all the code including padding
  CodeSegments* code_segment_info = (CodeSegments*)calloc(1,sizeof(CodeSegments));
  code_segment_info->segs = (CodeSegment**)calloc(100,sizeof(CodeSegment*));
  // Get Entry Point
  code_segment_info->entry_point = file_header->e_entry;
  for (int i = 0; i < file_header->e_phnum; i++){
    if (i > 90){
      assert("Handle Realloc of segment points list\n"== 0);
    }
    int offset = file_header->e_phoff + (i * file_header->e_phentsize);
    program_header_entry = (ElfProgHdr*)((char*)file_header + offset);
    if (program_header_entry->p_type != 1)
      continue;
    // Get Base Address
    if (i == 1) {
      code_segment_info->base_address = program_header_entry->p_vaddr;
    }
    // allocate a segment for each segment
    CodeSegment* segment = (CodeSegment*)calloc(1,sizeof(CodeSegment));
    segment->virtual_address = program_header_entry->p_vaddr;
    segment->size = program_header_entry->p_filesz;
    segment->raw_data = (char*)calloc(1,sizeof(char) * program_header_entry->p_filesz);
    code_segment_info->total_size += (program_header_entry->p_memsz + program_header_entry->p_vaddr);
    int start = program_header_entry->p_offset;
    int end = start + program_header_entry->p_filesz;
    int off = end - start;
    memcpy(segment->raw_data,&elf_data[start],off);
    //printf("%s\n",segment->raw_data);
    code_segment_info->segs[code_segment_info->count] = segment;
    code_segment_info->count++;
    //printf("Loaded Segment 0x%llx\n",segment->virtual_address);
  }
    return code_segment_info;
}


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