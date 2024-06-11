
#ifndef CRASH_HEADER 

#include <stdint.h>
#include <unordered_map>
#define CRASH_HEADER


typedef struct crash_map_t{
    uint64_t unique_crashes;
    uint64_t prev_unique_crashes;
    uint64_t crashes;
} CrashMap;


typedef bool (crash_callback)(CrashMap*);

#endif