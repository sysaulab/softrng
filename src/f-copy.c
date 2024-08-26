/*  Copyright 2023, All rights reserved, Sylvain Saucier
    sylvain@sysau.com
    Distributed under Affero GNU Public Licence version 3
    Commercial licence available upon request */

#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv)
{
    if(argc < 2)
    {
        fprintf(stderr, "srf_fwrite: FILE NAME IS MISSING.\n");
        exit(EXIT_FAILURE);
    }
    if(argc > 2)
    {
        fprintf(stderr, "srf_fwrite: ONLY 1 FILE IS ALLOWED.\n");
        exit(EXIT_FAILURE);
    }

    int readed = 0;
    int piped = 0;
    int forked = 0;

    char* buffer = calloc(_SSRNG_BUFSIZE, sizeof(char));
    if(buffer == NULL)
    {
        fprintf(stderr, "srf_fwrite: OUT OF MEMORY\n");
        exit(EXIT_FAILURE);
    }

    FILE* output = fopen(argv[1], "w");
    if(output == NULL)
    {
        fprintf(stderr, "srf_fwrite: CANNOT OPEN FILE\n");
        exit(EXIT_FAILURE);
    }

    while((readed = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin)))
    {
        forked = fwrite(buffer, sizeof(char), readed, output);
        piped = fwrite(buffer, sizeof(char), readed, stdout);
        if(readed != forked)
        {
            fprintf(stderr, "srf_fwrite: OUTPUT DISCREPENCIES in the forked data.\n");
            exit(EXIT_FAILURE);
        }
        if(readed != piped)
        {
            fprintf(stderr, "srf_fwrite: OUTPUT DISCREPENCIES in the piped data.\n");
            exit(EXIT_FAILURE);
        }
    }

    fclose(output);
    free(buffer);

    return EXIT_SUCCESS;
}
