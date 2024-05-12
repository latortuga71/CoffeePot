#ifndef ELF_HEADER

#define ELF_HEADER


#include <stdint.h>
#include <vector>
#include <stdlib.h>
#include <string.h>

typedef struct elf_file_hdr_t {
  unsigned char e_ident[16];
  uint16_t e_type;
  uint16_t e_machine;
  uint32_t e_version;
  uint64_t e_entry;
  uint64_t e_phoff;
  uint64_t e_shoff;
  uint32_t e_flags;
  uint16_t e_ehsize;
  uint16_t e_phentsize;
  uint16_t e_phnum;
  uint16_t e_shentsize;
  uint16_t e_shnum;
  uint16_t e_shstrndx;
} ElfFileHdr;

typedef struct elf_program_hdr_t {
  uint32_t p_type;
  uint32_t p_flags;
  uint64_t p_offset;
  uint64_t p_vaddr;
  uint64_t p_addr;
  uint64_t p_filesz;
  uint64_t p_memsz;
  uint64_t p_align;
} ElfProgHdr;

typedef struct code_segment_t {
  uint64_t base_address;
  uint64_t total_size;
  char* raw_data;
} CodeSegment;


CodeSegment* parse_elf_segments(char *, size_t);

#endif