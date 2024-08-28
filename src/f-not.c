#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    int readed = 0;
    int piped = 0;
    char buffer[_SSRNG_BUFSIZE];

    while((readed = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, stdin))) {
        for(int x = 0; x < _SSRNG_BUFSIZE; x++){
            buffer[x] = ~buffer[x];
        }
        piped = fwrite(&buffer, sizeof(char), readed, stdout);
        if(readed != piped) {
            fprintf(stderr, "f-not: discrepency in piped data.\n");
            exit(EXIT_FAILURE);
        }
    }

    return EXIT_SUCCESS;
}
