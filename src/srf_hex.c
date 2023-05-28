/**
* @file hex.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Pipe filter : stdin (binary) to stdout (hex) */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv){
    uint8_t* buffer = malloc(_SSRNG_BUFSIZE);
    while(1)
    {
        uint64_t buf_received = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, stdin);

        if(!buf_received)
            break;

        for( int x = 0; x < buf_received / 8; x++ )
            fprintf(stdout, "%016llx", ((uint64_t*)buffer)[x]);

        for( int y = (buf_received / 8) * 8; y < buf_received % 8; y++ )
            fprintf(stdout, "%02x", buffer[y]);
    }
    free(buffer);
    return EXIT_SUCCESS;
}
