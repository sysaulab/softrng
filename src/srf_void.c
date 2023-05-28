/*  Copyright 2023, All rights reserved, Sylvain Saucier
    sylvain@sysau.com
    Distributed under Affero GNU Public Licence version 3
    Commercial licence available upon request */

#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv)
{
    char* buffer = calloc(_SSRNG_BUFSIZE, sizeof(char));
    if(buffer == NULL) exit(EXIT_FAILURE);
    while(fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin));
    free(buffer);
    return EXIT_SUCCESS;
}