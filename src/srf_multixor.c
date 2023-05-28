/**
* @file keep.c
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
    int width = argc - 1;
    int readed;
    int min_readed;

    uint64_t* buffers = malloc(_SSRNG_BUFLEN * width * sizeof(uint8_t));
    if(!buffers) exit(EXIT_FAILURE);

    FILE** inputs = malloc(width * sizeof(FILE*));
    if(!inputs) exit(EXIT_FAILURE);

    //open the pipes
    for(int x = 0; x < width; x++)
    {
        inputs[x] = popen(argv[x+1], "r");
        if(!inputs[x]) exit(EXIT_FAILURE);
    }

    while(1)
    {
        min_readed = 0x0fffffff;

        //read from the pipes
        for(int x = 0; x < width; x++)
        {
            readed = fread(&(buffers[x*_SSRNG_BUFLEN]), sizeof(uint64_t), _SSRNG_BUFLEN, inputs[x]);
            if(!readed) exit(EXIT_SUCCESS);
            min_readed = (readed < min_readed) ? readed : min_readed;
        }

        //XOR buffers
        for(int x = _SSRNG_BUFLEN; x < _SSRNG_BUFLEN * width; x += _SSRNG_BUFLEN)
            for(int y = 0; y < _SSRNG_BUFLEN; y++)
                buffers[y] ^= buffers[x+y];

        if(min_readed == 0) exit(EXIT_SUCCESS);

        //output buffers
        int readed = fwrite(&buffers[0], sizeof(uint64_t), min_readed, stdout);
        if(readed != min_readed) 
            exit(EXIT_FAILURE);
    }
    free(buffers);
    free(inputs);
    return EXIT_SUCCESS;
}