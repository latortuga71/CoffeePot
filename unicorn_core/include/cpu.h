#ifndef CPU_HEADER

#define CPU_HEADER 

#include <stdint.h>
#include <vector>
#include <unordered_map>


typedef struct cpu_t{
    uint64_t pc;
    uint64_t stack_pointer;
    uint64_t x_reg[32];
    uint64_t f_reg[32];
} CPU;




#endif