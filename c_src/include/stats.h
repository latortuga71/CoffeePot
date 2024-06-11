#ifndef STATS_HEADER

#include <stdint.h>
#include <ctime>
#include <stdio.h>

#define STATS_HEADER 

typedef struct stats_t {
    uint64_t cases;
    std::time_t start_time;
} Stats ;


void display_stats(Stats* stats);

#endif