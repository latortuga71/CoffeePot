#ifndef COVERAGE_HEADER

#include <stdint.h>
#include <set>
#define COVERAGE_HEADER 


typedef struct coverage_map_t {
    std::set<uint64_t>*hashes;
    uint64_t unique_branches_taken;
    uint64_t branches_taken;
} CoverageMap;


typedef bool (coverage_callback)(CoverageMap*,uint64_t src, uint64_t dst);


// Coverage Callback
/*
Generally Should Hash Source & Destination Addresses 
Check If In Map
If 
    In Map Increment Hit Count?
Else
    Add To Map Indicate New Coverage Was Hit To Emulator?

*/

#endif