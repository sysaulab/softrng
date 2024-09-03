#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv)
{
    int offset = 0;
    if(argc < 2) {
        fprintf(stderr, "s-file: FILE NAME IS MISSING\n");
        exit(EXIT_FAILURE);
    }
    if(argc > 2) {
        offset = atoi(argv[2]);
    }

    FILE* input = fopen(argv[1], "r");
    if(!input) {
        fprintf(stderr, "s-file: cannot open file\n");
        exit(EXIT_FAILURE);
    }
    fseek(input, offset, SEEK_SET);

    char buffer[_SSRNG_BUFSIZE];
    int readed = 0;
    int width = argc - 1;

    while((readed = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, input))) {
        int y = fwrite(&buffer, sizeof(char), readed, stdout);
        if((y != readed)) {
            fprintf(stderr, "s-file: output discrepencies.\n");
            exit(EXIT_FAILURE);
        }
    }

    fclose(input);
    return EXIT_SUCCESS;
}
