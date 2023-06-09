/**
* @file srf_bench.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Utility to monitor bandwidth of piped data between commands */

#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include "common.h"
#include "sr_config.h"

double start = 0;
uint64_t bytes_written = 0;

/**
* @brief Main Thread
* @author Sylvain Saucier
* Main thread.
*/
int main(int argc, char** argv)
{
    char* buffer = calloc(_SSRNG_BUFSIZE, sizeof(char));
    while(1)
    {   
        fwrite(buffer, sizeof(char), _SSRNG_BUFSIZE, stdout);
    }
    free(buffer);
    return EXIT_SUCCESS;
}