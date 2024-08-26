#include <stdio.h>
#include <string.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    if(argc < 2) {
        fprintf(stderr, "srf_fork: sub-chain missing.\n");
        exit(EXIT_FAILURE);
    }

    int readed = 0;
    int piped = 0;
    int forked = 0;
    char buffer[_SSRNG_BUFSIZE];
    char command[4097];
    strncpy(command, argv[1], 256 );
    for (int x = 2; x < argc; x++) {
        strncpy(command, " ", 1 );
        strncpy(command, argv[x], 256 );
    }
    FILE* output = popen(command, "w");
    if(output == NULL) {
        fprintf(stderr, "srf_fork: cannot open pipe\n");
        exit(EXIT_FAILURE);
    }

    while((readed = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin))) {
        forked = fwrite(buffer, sizeof(char), readed, output);
        piped = fwrite(buffer, sizeof(char), readed, stdout);
        if(readed != forked) {
            fprintf(stderr, "srf_fwrite: OUTPUT DISCREPENCIES in the forked data.\n");
            exit(EXIT_FAILURE);
        }
        if(readed != piped) {
            fprintf(stderr, "srf_fwrite: OUTPUT DISCREPENCIES in the piped data.\n");
            exit(EXIT_FAILURE);
        }
    }

    fclose(output);
    return EXIT_SUCCESS;
}
