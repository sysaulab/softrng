/**
* @file seedy.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Pure ICM seed generator */

#include <stdio.h>
#include <stdlib.h>
#include <string.h>

#include "libicm.h"
#include "common.h"
#include "sr_config.h"

#ifndef _SRG_PRODUCTION
void fillbuffer(icm_state_t* state, uint64_t* buffer, uint64_t buffer_size, uint64_t first){
    if(!first){
        icm_fill64(state, buffer, buffer_size);}
    else{
        for(uint64_t y = 0; y < buffer_size; y++){
            for(int x = 0; x < _ICM_MAX_THREADS; x++) state->nodes[x] = 0;
            icm_fill64(state, buffer + y, 1);}}}
#endif

int main(int argc, const char * argv[]){
    uint64_t buffer[_SSRNG_BUFLEN];

#ifndef _SRG_PRODUCTION
    uint32_t cfg_debug = 0;
    uint32_t cfg_reset = 0;
    for( int x = 1; x < argc; x++ ){
        if(!strcmp("debug", argv[x])) cfg_debug = 1;
        else if(!strcmp("reset", argv[x])) cfg_reset = 1;
        else{
            printf("Distributed under Affero GNU Public Licence version 3\n\n");
            printf("usage : seed [debug reset] | output\n");
            return EXIT_FAILURE;}}
#endif
    icm_state_t state;
    icm_init(&state);
#ifdef _SSRNG_EXPERIMENTAL //Enable in libicm.h. Output from library, thread function.
    fprintf(stderr, "!!! WARNING !!! Experimental features in use\n");
    icm_join(&state);
#else
    while(1){
#ifndef _SRG_PRODUCTION
        fillbuffer(&state, buffer, _SSRNG_BUFLEN, cfg_reset);
#else
        icm_fill(&state, buffer, _SSRNG_BUFLEN);
#endif
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdout);
#ifndef _SRG_PRODUCTION
        if(cfg_debug){
            for( int x = 0; x < _ICM_MAX_THREADS; x++) fprintf(stderr, "\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b");
            for( int x = 0; x < _ICM_MAX_THREADS; x++) fprintf(stderr, "%016llx", state.nodes[x]);}
#endif
    }
#endif
    return EXIT_SUCCESS;}