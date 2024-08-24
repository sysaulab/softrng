/**
* @file srf_void.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Keep filter, relay at most n bytes from stdin to stdout */

#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv)
{
    char* buffer = calloc(_SSRNG_BUFSIZE, sizeof(char));
    if(buffer == NULL)
    {
        fprintf(stderr, "XOR IS OUT OF MEMORY\n");
        exit(EXIT_FAILURE);
    }

    while(fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin))
    {}

    free(buffer);
    return EXIT_SUCCESS;
}
