#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv){
    uint8_t buffer[_SSRNG_BUFSIZE];
    while(1) {
        uint64_t buf_received = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, stdin);
        if(buf_received == 0)
            break;
        for( int y = 0; y < buf_received; y++ ) 
            fprintf(stdout, "%02x", buffer[y]);
    }
    return EXIT_SUCCESS;
}
