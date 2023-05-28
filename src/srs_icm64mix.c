/**
* @file seedier.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* ICM seed mixer */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "libicm.h"
#include "sr_config.h"

int main(int argc, const char * argv[])
{
    uint64_t* buffer = malloc(_SSRNG_BUFLEN);
    icm_state_t state;
    icm_init(&state);
    while(1)
    {
        fread(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdin);
        icm_mix64(&state, buffer, _SSRNG_BUFLEN);
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdout);
    }
    free(buffer);
    icm_stop(&state);
    return EXIT_SUCCESS;
}
    