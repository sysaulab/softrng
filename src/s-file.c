#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv)
{
    if(argc < 2) {
        fprintf(stderr, "srf_fread: FILE NAME IS MISSING\n");
        exit(EXIT_FAILURE);
    }
    if(argc > 2) {
        fprintf(stderr, "srf_fread: ONLY 1 FILE IS ALLOWED\n");
        exit(EXIT_FAILURE);
    }

    FILE* input = fopen(argv[1], "r");
    if(!input) {
        fprintf(stderr, "srs_fread: cannot open file\n");
        exit(EXIT_FAILURE);
    }

    char buffer[_SSRNG_BUFSIZE];
    int readed = 0;
    int width = argc - 1;

    while((readed = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, input))) {
        int y = fwrite(&buffer, sizeof(char), readed, stdout);
        if((y != readed)) {
            fprintf(stderr, "srs_fread: output discrepencies.\n");
            exit(EXIT_FAILURE);
        }
    }
    fclose(input);
    return EXIT_SUCCESS;
}
