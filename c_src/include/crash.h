
#ifndef CRASH_HEADER 

#include <stdint.h>
#include <unordered_map>
#define CRASH_HEADER


typedef struct crash_map_t{
    std::unordered_map<uint64_t,uint64_t>hashes;
    uint64_t unique_crashes;
    uint64_t crashes;
} CrashMap;


typedef bool (crash_callback)(CrashMap*);


// Crash Callback
/*
Generally Should Hash crash PC

Check If In Map
If 
    In Map Increment Hit Count?
Else
    Add To Map Indicate New Coverage Was Hit To Emulator?

bool used to tell emulator that new crash was found?
*/

#endif