/*  Copyright 2023, All rights reserved, Sylvain Saucier
    sylvain@sysau.com
    Distributed under Affero GNU Public Licence version 3
    Other licences available upon request */

#include <pthread.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

#define  _MAX_THREADS 4

void* prng(void* raw)
{
    uint64_t* buffer = malloc(_SSRNG_BUFLEN);
    uint64_t* pool = calloc(4*65536, sizeof(uint64_t));
    if(!pool) exit(EXIT_FAILURE);
    uint64_t index = 0;
    uint16_t* indexes = (uint16_t*)&index;

    fread( pool, sizeof(uint64_t), 4 * 256 * 256, stdin);

    while(1)
    {
        for( uint64_t y = 0; y < _SSRNG_BUFLEN; y++ )
        {
            buffer[y] = (pool[indexes[0]] ^ pool[indexes[1]]) ^ (pool[indexes[2]] ^ pool[indexes[3]]);
            index = index + 7776210437768060567ULL;
        }
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdout);
    }
    free(buffer);
    return NULL;
}

int main(int argc, char** argv)
{
    int width = (argc > 1 ? atoi(argv[1]) : 1);
    width = width < 1 ? 1 : width;
    width = width >  _MAX_THREADS ?  _MAX_THREADS : width;
    pthread_t threads[ _MAX_THREADS];

    for (int x = 0; x < width; x++)
        pthread_create(&threads[x], NULL, prng, NULL);

    for (int x = 0; x < width; x++)
        pthread_join(threads[x], NULL);

    return EXIT_SUCCESS;
}

