#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    uint64_t hash = 0;
    if ( argc < 2 ) while(1) fwrite(&hash, sizeof(hash), 1, stdout);
    int length = strlen(argv[1]);
    while(1) {
        for ( int x = 0; x < length; x++ ) {
            hash = (hash << 7) | (hash >> (64-7));
            hash = hash ^ argv[1][x];
        }
        if ( !fwrite(&hash, sizeof(hash), 1, stdout) ) {
            fprintf(stderr, "s-password: write error.");
            exit(EXIT_FAILURE);
        }
    }
    return EXIT_SUCCESS;
}
