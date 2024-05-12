
#ifndef MMU_HEADER

#define MMU_HEADER 

#include <stdint.h>
#include <vector>
#include <map>
#include <tuple>

typedef struct segment_range_t {
    uint64_t start;
    uint64_t end;
} SegmentRange;

typedef struct memory_segment_t {
    uint8_t* data;
    size_t data_size;
    SegmentRange range;
    //... permissions etc
    //.. total_size size 
} Segment;

typedef struct mmu_t {
    Segment* virtual_memory;
    uint64_t segment_count;
    uint64_t segment_capacity;
    uint64_t next_allocation_base;
} MMU;


#endif