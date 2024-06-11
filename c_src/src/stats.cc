#include "stats.h"




void display_stats(Stats* stats) {
  std::time_t now = std::time(0);
  std::time_t elapsed = now - stats->start_time;
  float cases_per_second = (float)stats->cases / (float)elapsed;
  printf("Elapsed %'9.2f ::: Cases %'lu ::: Coverage %'lu FuzzCasesPerSecond %'9.2f\n",(float)elapsed, stats->cases, stats->unique_branches,(float)cases_per_second);
}