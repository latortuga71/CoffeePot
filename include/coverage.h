#ifndef COVERAGE_HEADER

#include <stdint.h>
#include <set>
#define COVERAGE_HEADER 


typedef struct coverage_map_t {
    std::set<uint64_t>*hashes;
    uint64_t unique_branches_taken;
    uint64_t previous_unique_branches_taken;
    uint64_t branches_taken;
} CoverageMap;


typedef bool (coverage_callback)(CoverageMap*,uint64_t src, uint64_t dst);

#endif