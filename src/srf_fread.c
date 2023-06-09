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
        fprintf(stderr, "srf_fread: FILE NAME IS MISSING\n");
        exit(EXIT_FAILURE);
    }
    if(argc > 2)
    {
        fprintf(stderr, "srf_fread: ONLY 1 FILE IS ALLOWED\n");
        exit(EXIT_FAILURE);
    }

    FILE* input = fopen(argv[1], "r");
    if(!input)
    {
        fprintf(stderr, "srf_fread: CANNOT OPEN FILE\n");
        exit(EXIT_FAILURE);
    }

    char* buffer = calloc(_SSRNG_BUFSIZE, sizeof(char));
    if(buffer == NULL)
    {
        fprintf(stderr, "srf_fread: OUT OF MEMORY\n");
        exit(EXIT_FAILURE);
    }

    int readed = 0;
    int width = argc - 1;

    while((readed = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, input)))
    {
        int y = fwrite(buffer, sizeof(char), readed, stdout);
        if((y^readed)) //by XOR ing readed and y we enter only if they are different.
        {
            fprintf(stderr, "srf_fread: OUTPUT DISCREPENCIES.\n");
            exit(EXIT_FAILURE);
        }
    }
    fclose(input);
    free(buffer);
    return EXIT_SUCCESS;
}
