#include "stats.h"




void display_stats(Stats* stats,Corpus* corpus) {
  std::time_t now = std::time(0);
  std::time_t elapsed = now - stats->start_time;
  float cases_per_second = (float)stats->cases / elapsed;
  printf("Crashes %d ::: Corpus %d ::: Elapsed %'9.2f ::: Cases %'lu ::: Coverage %'lu FuzzCasesPerSecond %'9.2f\n",stats->crashes,corpus->count,(float)elapsed, stats->cases, stats->unique_branches,(float)cases_per_second);
	//fmt.Printf("INFO: Crashes %d Iterations %d Coverage %d/%d %2f Cases Per Second %f Seconds %f Hours %f Corpus %d\n", s.Crashes, s.FuzzCases, s.BreakPointsHit, s.TotalBreakPoints, percent, float64(s.FuzzCases)/elapsed.Seconds(), elapsed.Seconds(), elapsed.Hours(), s.Corpus.CorpusCount)
}