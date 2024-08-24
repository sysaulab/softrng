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
#include <string.h>
#include "sr_config.h"



int main(int argc, char** argv)
{
    fprintf(stderr, "srs_password\n");
    uint64_t hash = 0;
    if ( argc < 2 ) while(1) fwrite(&hash, sizeof(hash), 1, stdout);
    int length = strlen(argv[1]);
    fprintf(stderr, "length:%i\n", length);
    while(1) {
        for ( int x = 0; x < length; x++ ) {
            hash = (hash << 7) | (hash >> (64-7));
            hash = hash ^ argv[1][x];
//            fprintf(stderr, "hash_building:%016llx\n", hash);
        }
        if ( !fwrite(&hash, sizeof(hash), 1, stdout) )
        {
            fprintf(stderr, "s-password: Write error.");
            exit(EXIT_FAILURE);
        }
    }
    return EXIT_SUCCESS;
}
