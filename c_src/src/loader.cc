#include "loader.h"
#include <stdio.h>

CodeSegment* parse_elf_segments(char *elf_data,size_t file_size) {
  ElfFileHdr *file_header = (ElfFileHdr *)elf_data;
  //printf("ELF Header -> %s\n", file_header->e_ident);
  //printf("ELF Entry -> 0x%x\n", file_header->e_entry);
  //printf("ELF LOAD SEGMENTS -> %d\n", file_header->e_phnum);
  uint64_t ph_offset = file_header->e_phoff;
  uint64_t ph_size = file_header->e_phentsize;
  int offset = 0;
  ElfProgHdr* program_header_entry = NULL;
  // Prepare buffer that will contain all the code including padding
  CodeSegment* code_segment_info = (CodeSegment*)calloc(1,sizeof(CodeSegment));
  code_segment_info->entry_point = file_header->e_entry;
  code_segment_info->raw_data = (char*)calloc(1,file_size);
  code_segment_info->total_size = 0x0;
  code_segment_info->base_address = 0x0;
  for (int i = 0; i < file_header->e_phnum; i++){
    int offset = file_header->e_phoff + (i * file_header->e_phentsize);
    program_header_entry = (ElfProgHdr*)((char*)file_header + offset);
    if (program_header_entry->p_type != 1)
      continue;
    /*
    printf("type 0x%x \n",program_header_entry->p_type);
    printf("Virtual Address 0x%x \n",program_header_entry->p_addr);
    printf("Size 0x%x\n",program_header_entry->p_memsz);
    printf("File Sz 0x%x\n",program_header_entry->p_filesz);
    */
    // handle copying the data to 1 buffer this will be our text section in memory;
    code_segment_info->total_size += program_header_entry->p_memsz; // <- look here if you have issues
    int start = program_header_entry->p_offset;
    int end = start + program_header_entry->p_filesz;
    int off = end - start;
    if (i == 1) {
      code_segment_info->base_address = program_header_entry->p_vaddr;
    }
    memcpy(code_segment_info->raw_data,&elf_data[start],off);
  }
  // add values to struct
  //printf("Code Segment Entry 0x%x\n",code_segment_info->entry_point);
  //printf("Code Segment Info Base Address 0x%x\n",code_segment_info->base_address);
  //printf("Code Segment Info Size %d\n",code_segment_info->total_size);
  return code_segment_info;
}
