
#ifndef CRASH_HEADER 

#include <stdint.h>
#include <unordered_map>
#define CRASH_HEADER
#include "corpus.h"


typedef struct crash_map_t{
    uint64_t unique_crashes;
    uint64_t prev_unique_crashes;
    uint64_t crashes;
    char* crashes_dir;
} CrashMap;


typedef bool (crash_callback)(CrashMap*,uint64_t,FuzzCase*);

#endif