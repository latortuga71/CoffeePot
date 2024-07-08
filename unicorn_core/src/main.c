#include <unicorn/unicorn.h>
#include <assert.h>
#include <stdio.h>
#include <unistd.h>
#include <sys/uio.h>
#include <sys/mman.h>
#include "loader.h"

// code to be emulated
#define RISCV_CODE64 "\x93\x08\xd0\x05" \
"\x13\x05\x40\x00" \
"\x73\x00\x00\x00"
//#define RISCV_CODE64 "\x13\x05\x10\x00\x93\x85\x05\x02"
// memory address where emulation starts
#define ADDRESS 0x1000000
#define STACK_SIZE 1024*1024*8;


typedef struct Allocations {
  uint64_t next_allocation_base;
};

struct Allocations global_allocs;


uint64_t align_address_up(uint64_t address){
  return (address & ~(4 * 1024)) + (4 * 1024);
}


// Hook Basic Blocks To Track Coverage
static void hook_block(uc_engine *uc, uint64_t address, uint32_t size,
                       void *user_data)
{
    printf(">>> Tracing basic block at 0x%" PRIx64 ", block size = 0x%x\n",
           address, size);
}

/*
static void hook_code(uc_engine *uc, uint64_t address, uint32_t size,
                      void *user_data)
{
    printf(">>> Tracing instruction at 0x%" PRIx64
           ", instruction size = 0x%x\n",
           address, size);
    uint64_t pc = 0x0;
    uc_reg_read(uc,UC_RISCV_REG_PC,&pc);
    printf("0x%llx\n",pc);
}
*/


uint64_t align_size_up(uint64_t sz){
  uint64_t multiple = sz / 0x1000;
  return ((4*1024) * multiple) + 0x1000;
}

// Hook Interrupts to handle syscalls
static void hook_syscall(uc_engine *uc, uint64_t address, uint32_t size,
                      void *user_data)
{
    uc_err err;
    //printf("System Interrupt!\n");
    uint64_t x17 = 0x0;
    uint64_t x10 = 0x0;
    uint64_t x11 = 0x0;
    uint64_t x12 = 0x0;
    uc_reg_read(uc,UC_RISCV_REG_A7,&x17);
    uc_reg_read(uc,UC_RISCV_REG_A0,&x10);
    uc_reg_read(uc,UC_RISCV_REG_A1,&x11);
    uc_reg_read(uc,UC_RISCV_REG_A2,&x12);
    switch (x17) {
      case 0xd7: {
        printf("syscall munmap 0x%llx 0x%llx 0x%llx\n",x10,x11);
        uint64_t result = munmap(x10,x11);
        uc_reg_write(uc,UC_RISCV_REG_A0,&result);
        return;
      }
      case 0xde: {
        printf("syscall mmap 0x%llx 0x%llx 0x%llx\n",x10,x11,x12);
        /*
        stack_base = mmap(NULL,stack_size,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
        if (stack_base == MAP_FAILED){
          fprintf(stderr,"ERROR: Failed to mmap stack :::\n");
          exit(-1);
        }*/
        if (x11 == 0){
          x10 = -1;
          uc_reg_write(uc,UC_RISCV_REG_A0,&x10);
          return;
        }
        uint64_t base;
        // TODO HANDLE MEMORY PERMISSIONS
        //uint64_t aligned_address = align_address_up(x10);
        if (x10 == 0) {
          printf("attempting to alloc here 0x%llx\n",align_address_up(global_allocs.next_allocation_base));
          printf("size aligned 0x%llx 0x%llx\n",x11,align_size_up(x11));
          base = mmap(NULL,align_size_up(x11),PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
          if (base == MAP_FAILED){
            fprintf(stderr,"ERROR: Failed to mmap stack :::\n");
            exit(-1);
          }
          //err = uc_mem_map_ptr(uc,stack_base,stack_size,UC_PROT_READ | UC_PROT_WRITE,stack_base);
          err = uc_mem_map_ptr(uc,base,align_size_up(x11),UC_PROT_ALL,base);
          if (err){
            fprintf(stderr ,"ERROR: uc_mem_map_ptr mmap emulation failed %s\n",uc_strerror(err));
            exit(-1);
          }
          printf("mmap -> start 0x%llx -> end 0x%llx sz 0%llx\n",base,global_allocs.next_allocation_base,x11);
          getchar();
          uc_reg_write(uc,UC_RISCV_REG_A0,&base);
          return;
        }
        printf("todo handle no address mmap\n");
        exit(-1);
      }
      case 0xd6: {
        printf("syscall sbrk(0x%llx)\n",x10);
        //void* result = sbrk((intptr_t)x10);
        //printf("sbrk result = 0x%llx\n",(uint64_t)result);
        uint64_t break_addr = (uint64_t)-1;
        uc_reg_write(uc,UC_RISCV_REG_A0,&break_addr);
        return;
      }
      case 0x5e: {
        //printf("Exit Syscall Code 0x%llx!\n",x10);
        exit(x10);
        return;
      }
      case 0x60: {
        //printf("syscall -> set_tid_address\n");
        uint64_t pid = (uint64_t)getpid();
        uc_reg_write(uc,UC_RISCV_REG_A0,&pid);
        return;
      }
      case 0x1d: {
        //printf("syscall -> ioctl\n");
        uc_reg_write(uc,UC_RISCV_REG_A0,&x10);
        return;
      }
      case 0xe2: {
        //printf("syscall -> mprotect%s","\n");
        x10 = 0;
        uc_reg_write(uc,UC_RISCV_REG_A0,&x10);
        return;
      }
      case 0x42:{
        //printf("syscall -> writev%s","\n");
        uint64_t file_descriptor = x10;
        struct iovec* iovec_ptr = (struct iovec*)malloc(sizeof(struct iovec) * x12);
        err = uc_mem_read(uc,x11,(void*)iovec_ptr,sizeof(struct iovec) * x12);
        if (err){
          fprintf(stderr,"ERROR: Failed to uc_mem_read 0x%llx ::: %s \n",x11 ,uc_strerror(err));
          exit(-1);
        }
        int iovcnt = (int)x12;
        //printf("buffer address 0x%llx buffer length %d file_descriptor %d\n",iovec_ptr->iov_base,iovec_ptr->iov_len,file_descriptor);
        // Read Memory Buffer
        uint64_t write_count = 0;
        for (int i = 0; i < iovcnt; i++){
          //printf("iovec ptr -> 0x%llx\n",iovec_ptr);
          //printf("fd %d iovec ptr 0x%llx, count %d\n",x10,x11,x12);
          //printf("iovec data-> 0x%llx iovec data sz -> %d\n",iovec_ptr->iov_base,iovec_ptr->iov_len);
          // TODO dont copy memory into another buffer
          if (iovec_ptr->iov_base == 0){
              write_count += iovec_ptr->iov_len;
              iovec_ptr++;
              continue;
          }
          //void* buffer = vm_copy_memory(&emu->mmu,(uint64_t)(iovec_ptr->iov_base),iovec_ptr->iov_len);
          void* buffer = malloc(iovec_ptr->iov_len);
          err = uc_mem_read(uc, iovec_ptr->iov_base,buffer,iovec_ptr->iov_len);
          if (err){
            fprintf(stderr,"ERROR: Failed to uc_mem_read 0x%llx ::: %s \n",x11 ,uc_strerror(err));
            exit(-1);
          }
          // Write it to corresponding file descriptor
          if (file_descriptor == STDOUT_FILENO) {
              write(file_descriptor,buffer,iovec_ptr->iov_len);
          }
          write_count += iovec_ptr->iov_len;
          free(buffer);
          iovec_ptr++;
        }
        free(iovec_ptr);
        uc_reg_write(uc,UC_RISCV_REG_A0,&write_count);
        return;
      } 
      default:
        fprintf(stderr, "ERROR: Unknown Syscall 0x%llx\n",x17);
        assert("EXIT" == 0);
        return;
    }
}


/// HANDLE LOAD ELF
void load_code_segments_into_virtual_memory(uc_engine* uc ,CodeSegments* code){
  uc_err err;
  printf("0x%llx 0x%llx\n",code->base_address,code->total_size);
  uint64_t page = 4 * 1024;
  uint64_t aligned_address  = (code->total_size & ~(page-1)) + page;
  printf("0x%llx\n",(code->total_size & ~(page-1)) + page);
  err = uc_mem_map(uc,code->base_address,aligned_address,UC_PROT_ALL);
  if (err){
    fprintf(stderr,"ERROR: Failed to uc_mem_map ::: %s\n",uc_strerror(err));
    exit(-1);
  }
  global_allocs.next_allocation_base = code->base_address + aligned_address;
  fprintf(stderr,"DEBUG: Next Allocation Base -> 0x%llx\n",global_allocs.next_allocation_base);
  fprintf(stderr,"DEBUG: MMAP memory at 0x%llx sz 0x%llx\n",code->base_address,aligned_address);
  for (int i = 0; i < code->count; i++){
    err = uc_mem_write(uc,code->segs[i]->virtual_address,code->segs[i]->raw_data,code->segs[i]->size);
    if (err) {
      fprintf(stderr,"ERROR: Failed to uc_mem_write ::: %s\n",uc_strerror(err));
      exit(-1);
    }
    fprintf(stderr,"DEBUG: wrote 0x%llx bytes to 0x%llx\n",code->segs[i]->size,code->segs[i]->virtual_address);
  }
  fprintf(stderr,"DEBUG: load_segments_into_virtual_memory complete\n");
}

uint64_t load_elf_into_mem(const char* path, uc_engine* uc){
  CodeSegments* segments = parse_elf(path);
  uint64_t pc = segments->entry_point;
  load_code_segments_into_virtual_memory(uc,segments);
  delete_code_segments(segments);
  return pc;
}


uint64_t init_stack_virtual_memory(uc_engine* uc, int argc, char** argv){
  //uint64_t stack_size = (4 * 1024) * 0x7d;
  uint64_t stack_size = STACK_SIZE;
  uint64_t heap_size = (4 * 1024);
  uc_err err;
  char* stack_base = NULL;
  char* heap_base = NULL;

  stack_base = mmap(NULL,stack_size,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
  if (stack_base == MAP_FAILED){
    fprintf(stderr,"ERROR: Failed to mmap stack :::\n");
    exit(-1);
  }

  heap_base = mmap(NULL,heap_size,PROT_READ|PROT_WRITE,MAP_PRIVATE|MAP_ANONYMOUS,-1,0);
  if (heap_base == MAP_FAILED){
    fprintf(stderr,"ERROR: Failed to mmap heap :::\n");
    exit(-1);
  }

  err = uc_mem_map_ptr(uc,stack_base,stack_size,UC_PROT_READ | UC_PROT_WRITE,stack_base);

  if (err){
    fprintf(stderr,"ERROR: Failed to uc_mem_map_ptr stack ::: %s\n",uc_strerror(err));
    exit(-1);
  }

  printf("STACK IS AT 0x%llx\n",stack_base + stack_size);
  printf("HEAP IS AT 0x%llx\n",heap_base);

  err = uc_mem_map_ptr(uc,heap_base,heap_size ,UC_PROT_READ | UC_PROT_WRITE,heap_base);
  if (err){
    fprintf(stderr,"ERROR: Failed to uc_mem_map_ptr heap ::: %s\n",uc_strerror(err));
    exit(-1);
  }
  uint64_t stack_end = stack_base + stack_size;
  uint64_t stack_pointer = stack_end - (0x200);

  stack_pointer -= 8;
  err = uc_mem_write(uc,stack_pointer,(const uint8_t*)"\x00\x00\x00\x00\x00\x00\x00\x00",8);
  if (err){
    fprintf(stderr,"ERROR: Failed to init stack ::: %s\n",uc_strerror(err));
    exit(-1);
  }

  stack_pointer -= 8;
  err = uc_mem_write(uc,stack_pointer,(const uint8_t*)"\x00\x00\x00\x00\x00\x00\x00\x00",8);
  if (err){
    fprintf(stderr,"ERROR: Failed to init stack ::: %s\n",uc_strerror(err));
    exit(-1);
  }

  stack_pointer -= 8;
  err = uc_mem_write(uc,stack_pointer,(const uint8_t*)"\x00\x00\x00\x00\x00\x00\x00\x00",8);
  if (err){
    fprintf(stderr,"ERROR: Failed to init stack ::: %s\n",uc_strerror(err));
    exit(-1);
  }
  // write string to heap base
  err = uc_mem_write(uc,heap_base,(const uint8_t*)"\x41\x41\x41\x41\x41\x41\x41\x41",8);
  // write heap pointer to satck
  stack_pointer -= 8;
  char* buffer = &heap_base;
  err = uc_mem_write(uc,stack_pointer,buffer,8);
  if (err) {
    fprintf(stderr,"ERROR: Failed to uc_mem_write ::: %s\n",uc_strerror(err));
    exit(-1);
  }
  // write argc to the stack
  stack_pointer -= 8;
  err = uc_mem_write(uc,stack_pointer,(const uint8_t*)"\x01",1);
  if (err) {
    fprintf(stderr,"ERROR: Failed to uc_mem_write ::: %s\n",uc_strerror(err));
    exit(-1);
  }
  fprintf(stderr,"DEBUG: Stack Initialized\n");
  fprintf(stderr,"DEBUG: Heap Initialized\n");
  return stack_pointer;
}


int main(int argc, char **argv, char **envp)
{
  uc_engine *uc;
  uc_err err;
  uc_hook trace1, trace2,syshook;
  printf("Emulate risc-v64 code\n");
  // Initialize emulator in RISCV64 mode
  err = uc_open(UC_ARCH_RISCV, UC_MODE_RISCV64, &uc);
  if (err != UC_ERR_OK) {
    printf("Failed on uc_open() with error returned: %u\n", err);
    return -1;
  }
  //uint64_t pc = load_elf_into_mem("./hello_world_test",uc);
  uint64_t pc = load_elf_into_mem("./heap_test",uc);
  uint64_t sp = init_stack_virtual_memory(uc,argc,argv);
  // initialize machine registers
  /*
  uc_reg_write(uc, UC_RISCV_REG_A0, &x10);
  uc_reg_write(uc, UC_RISCV_REG_A7, &x17);
  // tracing all basic blocks with customized callback
  uc_hook_add(uc, &trace1, UC_HOOK_BLOCK, hook_block, NULL, 1, 0);
  // tracing all instruction
  //uc_hook_add(uc, &trace2, UC_HOOK_CODE, hook_code, NULL, 1, 0);
  */
  // emulate code in infinite time & unlimited instructions
  //err = uc_mem_write(uc,pc,RISCV_CODE64,sizeof(RISCV_CODE64));
  err = uc_reg_write(uc,UC_RISCV_REG_PC,&pc);
  if (err){
    fprintf(stderr,"DEBUG: Failed to set PC ::: %s\n",uc_strerror(err));
    return -1;
  }
  err = uc_reg_write(uc,UC_RISCV_REG_SP,&sp);
  if (err){
    fprintf(stderr,"DEBUG: Failed to set SP ::: %s\n",uc_strerror(err));
    return -1;
  }
  err = uc_reg_write(uc,UC_RISCV_REG_X2,&sp);
  if (err){
    fprintf(stderr,"DEBUG: Failed to set SP(x2) ::: %s\n",uc_strerror(err));
    return -1;
  }
  // hook syscalls
  err = uc_hook_add(uc, &syshook, UC_HOOK_INTR, hook_syscall, NULL, 1,0);
  if (err){
    fprintf(stderr,"DEBUG: Failed to set interrupt hook ::: %s\n",uc_strerror(err));
    return -1;
  }
  printf("Emulator Start 0x%llx\n",pc);
  /*
  char buffer[4];
  err = uc_mem_read(uc,pc,buffer,4);
  if (err) {
    printf("Failed on uc_emu_read() with error returned %u: %s\n",
      err, uc_strerror(err));
  }
  printf("-> 0x%llx\n",buffer[0]);
  */
  uint64_t gp;
  uint64_t s1;
  uint64_t steps = 0;
  // single step through the code
  for (;;){
    uc_reg_read(uc,UC_RISCV_REG_PC,&pc);
    err = uc_emu_start(uc, pc, pc + 12, 0, 1);
    if (err) {
      printf("Failed on uc_emu_start() with error returned %u: %s\n",
        err, uc_strerror(err));
        return -1;
    }
    uc_reg_read(uc,UC_RISCV_REG_SP,&sp);
    uc_reg_read(uc,UC_RISCV_REG_GP,&gp);
    uc_reg_read(uc,UC_RISCV_REG_S1,&s1);
    printf("PC -> 0x%llx\n",pc);
    printf("SP -> 0x%llx\n",sp);
    printf("GP -> 0x%llx\n",gp);
    printf("S1 -> 0x%llx\n",s1);
    /*
    steps++;
    if (steps > 5)
      break;
      */
  }
  uc_close(uc);

  return 0;
}

