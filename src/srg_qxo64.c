#include <pthread.h>
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    uint64_t buffer[_SSRNG_BUFLEN];
    uint64_t pool[4*65536];
    union {
        uint64_t u64;
        uint16_t u16[4];
    }
    index;
    index.u64 = 0;
    fread( &pool, sizeof(uint64_t), 4 * 256 * 256, stdin);
    while(1) {
        for( uint64_t y = 0; y < _SSRNG_BUFLEN; y++ ) {
            index.u64 = index.u64 + 7776210437768060567ULL;// *MUST* be prime and close to 2^63
            buffer[y] = 
            pool[(0*256*256)+index.u16[0]] ^ 
            pool[(1*256*256)+index.u16[1]] ^ 
            pool[(2*256*256)+index.u16[2]] ^ 
            pool[(3*256*256)+index.u16[3]];
        }
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdout);
    }

    return EXIT_SUCCESS;
}

