#include <stdio.h>
#include <string.h>
#include <stdint.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    char buffer[_SSRNG_BUFSIZE];
    uint64_t seed = 0;
    uint64_t buf_received = 0;
    char cmd[4096] = "";
    if(fread(&seed, sizeof(uint64_t), 1, stdin) == 0) {
        fprintf(stderr, "srt_rngoutput: error reading stdin");
    }
    sprintf(cmd, "RNG_output %s inf %016llx", argv[1], seed);
    FILE* child = popen(cmd, "r");
    if(child == NULL) {
        fprintf(stderr, "srt_rngoutput: error cannot open %s", cmd);
    }
    while(1) { 
        buf_received = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, child);
        if(buf_received != _SSRNG_BUFSIZE) {
            fprintf(stderr, "srt_rngoutput: error reading child. (%llu,%i)", buf_received, _SSRNG_BUFSIZE);
            break;
        }
        if(buf_received != fwrite(buffer, sizeof(char), _SSRNG_BUFSIZE, stdout)){
            fprintf(stderr, "srt_rngoutput: error writing stdout. (%llu,%i)", buf_received, _SSRNG_BUFSIZE);
            break;
        }
    }
    return EXIT_SUCCESS;
}
