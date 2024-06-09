#ifndef STATS_HEADER

#include <stdint.h>
#define STATS_HEADER 


typedef struct stats_t {
    uint64_t cases;
    float running_time_hours;
    float cases_per_second;
} Stats ;


#endif