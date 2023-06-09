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

#define _SSRNG_BUFLEN_LONG (_SSRNG_BUFLEN*32)
#define  _MAX_THREADS 4

inline uint64_t rot64(uint64_t i, int s)
{
    return (i << s) | (i >> (64 - s));
}

void* prng(void * raw)
{
    uint64_t* buffer = malloc(_SSRNG_BUFLEN_LONG);
    uint64_t prng_state[4][256];
    uint64_t index = 0;
    int64_t fresh = 0;
    uint8_t* indexes = (uint8_t*)&index;
    while(1)
    {
        if(fresh < 1)
        {
            fresh = 1099511627776;// 16TB, should be safe. 2^54 possible iterations
            fread( prng_state, sizeof(uint64_t), 4 * 256, stdin);
        }
        for( uint64_t y = 0; y < _SSRNG_BUFLEN_LONG; y++ )
        {
            uint64_t a = rot64(prng_state[0][indexes[0]], indexes[4] & 0b00111111);
            uint64_t b = rot64(prng_state[1][indexes[1]], indexes[5] & 0b00111111);
            uint64_t c = rot64(prng_state[2][indexes[2]], indexes[6] & 0b00111111);
            uint64_t d = rot64(prng_state[3][indexes[3]], indexes[7] & 0b00111111);
            buffer[y] = a ^ b ^ c ^ d;

            index = index + 7776210437768060567ULL;
            fresh--;
        }
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN_LONG, stdout);
    }
    free(buffer);
    return NULL;
}

int main(int argc, char** argv)
{
    int width = (argc > 1 ? atoi(argv[1]) : 1);
    width = width < 1 ? 1 : width;
    width = width >  _MAX_THREADS ?  _MAX_THREADS : width;
    pthread_t threads[_MAX_THREADS];

    for (int x = 0; x < width; x++)
        pthread_create(&threads[x], NULL, prng, NULL);

    for (int x = 0; x < width; x++)
        pthread_join(threads[x], NULL);

    return EXIT_SUCCESS;
}

