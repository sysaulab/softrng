#include <pthread.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

#define _SSRNG_BUFLEN_LONG (_SSRNG_BUFLEN*32)
#define rot64(i, s) ((i << s) | (i >> (64 - s)))

int main(int argc, char** argv) {
    uint64_t buffer[_SSRNG_BUFLEN_LONG];
    uint64_t prng_state[4][256];
    uint64_t index = 0;
    uint8_t* indexes = (uint8_t*)&index;
    fread(&prng_state, sizeof(uint64_t), 4*256, stdin);
    while(1) {
        for( uint64_t y = 0; y < _SSRNG_BUFLEN_LONG; y++ ) {
            index = index + 7776210437768060567ULL;
            buffer[y] = 
            rot64(prng_state[0][indexes[0]], indexes[1] % 64) ^ 
            rot64(prng_state[1][indexes[2]], indexes[3] % 64) ^
            rot64(prng_state[2][indexes[4]], indexes[5] % 64) ^
            rot64(prng_state[3][indexes[6]], indexes[7] % 64) ;
        }
        fwrite(&buffer, sizeof(uint64_t), _SSRNG_BUFLEN_LONG, stdout);
    }
    return EXIT_SUCCESS;
}

