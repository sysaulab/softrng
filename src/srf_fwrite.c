#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    if(argc != 2) {
        fprintf(stderr, "srf_fwrite: provide one file.\n");
        exit(EXIT_FAILURE);
    }

    int readed = 0;
    int piped = 0;
    int forked = 0;
    char buffer[_SSRNG_BUFSIZE];

    FILE* output = fopen(argv[1], "w");
    if(output == NULL) {
        fprintf(stderr, "srf_fwrite: cannot open file\n");
        exit(EXIT_FAILURE);
    }

    while((readed = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, stdin))) {
        forked = fwrite(&buffer, sizeof(char), readed, output);
        piped = fwrite(&buffer, sizeof(char), readed, stdout);
        if(readed != forked) {
            fprintf(stderr, "srf_fwrite: discrepency in forked data.\n");
            exit(EXIT_FAILURE);
        }
        if(readed != piped) {
            fprintf(stderr, "srf_fwrite: discrepency in piped data.\n");
            exit(EXIT_FAILURE);
        }
    }

    fclose(output);
    return EXIT_SUCCESS;
}
