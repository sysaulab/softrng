#include "sr_config.h"
#include <errno.h>
#include <stdint.h>
#include <stdio.h>
#include <string.h>
#include <time.h>

int main(int argc, char **argv) {
  uint64_t buffer[_SSRNG_BUFLEN];
  uint32_t buffer_low[_SSRNG_BUFLEN];
  uint32_t buffer_high[_SSRNG_BUFLEN];
  time_t now;
  time(&now);
  char name[1024];
  if (argc > 1)
    strcpy(name, argv[1]);
  else
    sprintf(name, "%li", now);
  int run_u01sl = 1;
  int run_u01sh = 1;
  int run_u01sb = 1;
  int run_pr64mt = 1;

  char path_u01s_low[1024] = ""; sprintf(path_u01s_low, "t-test-u01sc > %s_u01s_low.txt", name);
  char path_u01s_high[1024] = ""; sprintf(path_u01s_high, "t-test-u01sc > %s_u01s_high.txt", name);
  char path_u01s_both[1024] = ""; sprintf(path_u01s_both, "t-test-u01sc > %s_u01s_both.txt", name);
  char path_pr64mt[1024] = ""; sprintf(path_pr64mt, "t-test-pr64mt -tlmax 2G > %s_pr64mt_2g.txt", name);
  
  FILE *test_u01s_low = popen(path_u01s_low, "w");
  if (test_u01s_low == NULL) {
    fprintf(stderr, "Error: test_u01s_low %d\n", errno);
  }
  FILE *test_u01s_high = popen(path_u01s_high, "w");
  if (test_u01s_high == NULL) {
    fprintf(stderr, "Error: test_u01s_high %d\n", errno);
  }
  FILE *test_u01s_both = popen(path_u01s_both, "w");
  if (test_u01s_both == NULL) {
    fprintf(stderr, "Error: test_u01s_both %d\n", errno);
  }

  FILE *test_pr64mt = popen(path_pr64mt, "w");
  if (test_pr64mt == NULL) {
    fprintf(stderr, "Error: test_pr64mt %d\n", errno);
  }

  while (run_u01sl + run_u01sh + run_u01sb + run_pr64mt) {
    fflush(stdout);
    fflush(stdin);
    int raw_bytes_in = fread(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdin);

    if (run_u01sb) {
      if (fwrite(buffer, sizeof(uint64_t), raw_bytes_in, test_u01s_both) < raw_bytes_in) {
        run_u01sb = 0;
fprintf(stderr, "run_u01sb:done\n");
      }
    }

    if (run_pr64mt) {
      if (fwrite(buffer, sizeof(uint64_t), raw_bytes_in, test_pr64mt) < raw_bytes_in) {
        run_pr64mt = 0;
fprintf(stderr, "run_pr64mt:done\n");
      }
    }

  }

  return 0;

bad:
  fprintf(stderr, "NULL pointer");
}
