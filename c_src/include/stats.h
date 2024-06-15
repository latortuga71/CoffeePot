#ifndef STATS_HEADER

#include <stdint.h>
#include <ctime>
#include <stdio.h>
#include "corpus.h"

#define STATS_HEADER 

typedef struct stats_t {
    uint64_t crashes;
    uint64_t unique_branches;
    uint64_t cases;
    std::time_t start_time;
} Stats ;


void display_stats(Stats* stats,Corpus* corpus);

#endif