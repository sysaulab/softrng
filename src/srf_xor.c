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

int main(int argc, char** argv)
{
    if(argc < 2 || argc > 2)
    {
        fprintf(stderr, "XOR NEED 1 ARGUMENT\n");
        exit(EXIT_FAILURE);
    }
    int readed;
    int min_readed;

    uint64_t* buffers = malloc(_SSRNG_BUFLEN * sizeof(uint8_t));
    if(!buffers)
    {
        fprintf(stderr, "XOR IS OUT OF MEMORY\n");
        exit(EXIT_FAILURE);
    }

    FILE* input = popen(argv[1], "r");
    if(!input)
    {
        fprintf(stderr, "XOR CANNOT OPEN %s\n", argv[1]);
        exit(EXIT_FAILURE);
    }

    while(1)
    {
        min_readed = 0x0fffffff;

        readed = fread(&(buffers[0]), sizeof(uint64_t), _SSRNG_BUFLEN, stdin);
        min_readed = readed;

        readed = fread(&(buffers[_SSRNG_BUFLEN]), sizeof(uint64_t), _SSRNG_BUFLEN, input);
        min_readed = (readed < min_readed) ? readed : min_readed;

        if(!min_readed)
        {
            exit(EXIT_SUCCESS);
        }

        for(int y = 0; y < _SSRNG_BUFLEN; y++)
        {
            buffers[y] ^= buffers[_SSRNG_BUFLEN+y];
        }

        //output buffers
        int readed = fwrite(&buffers[0], sizeof(uint64_t), min_readed, stdout);
        if(readed ^ min_readed) 
        {
            fprintf(stderr, "XOR IS HAVING A PROBLEM WITH OUTPUT\n");
            exit(EXIT_FAILURE);
        }
    }
    free(buffers);
    pclose(input);
    return EXIT_SUCCESS;
}