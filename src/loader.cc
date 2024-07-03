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
