/**
* @file srf_xor.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Keep filter, relay at most n bytes from stdin to stdout */

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    if(argc < 2) {
        fprintf(stderr, "f-xor: must supply arguments\n");
        exit(EXIT_FAILURE);
    }
    int left_readed;
    int right_readed;
    uint64_t left_buffer[_SSRNG_BUFLEN];
    uint64_t right_buffer[_SSRNG_BUFLEN];
    char command[4097];
    strncpy(command, argv[1], 256 );
    for (int x = 2; x < argc; x++) {
        strncpy(command, " ", 1 );
        strncpy(command, argv[x], 256 );
    }
    FILE* input = popen(command, "r");
    if(!input) {
        fprintf(stderr, "f-xor: cannot open %s\n", argv[1]);
        exit(EXIT_FAILURE);
    }
    while(1) {
        left_readed = fread(&left_buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdin);
        right_readed = fread(&right_buffer, sizeof(uint64_t), _SSRNG_BUFLEN, input);
        if (left_readed != right_readed) {
            fprintf(stderr, "f-xor: read mismatch\n");
            exit(EXIT_FAILURE);
        }
        if(left_readed == 0) exit(EXIT_SUCCESS);
        for(int y = 0; y < _SSRNG_BUFLEN; y++) {
            left_buffer[y] ^= right_buffer[y];
        }
        int readed = fwrite(&left_buffer, sizeof(uint64_t), left_readed, stdout);
        if( readed != left_readed ) {
            fprintf(stderr, "f-xor: output problem\n");
            exit(EXIT_FAILURE);
        }
    }
    pclose(input);
    return EXIT_SUCCESS;
}
