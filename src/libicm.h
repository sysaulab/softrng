/**
* @file libicm.h
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* ICM library header */

#ifndef ___libicm_h
#define ___libicm_h

#include <pthread.h>
#include <stdint.h>
#include <stdio.h>
#include <unistd.h>
#include <time.h>
#include "sr_config.h"

#define _ICM_WARMUP         1000                    // Warmup after init in microseconds
#define _ICM_WAIT           100                      // Sleeping time in microseconds
#define _ICM_MAX_THREADS    3                       // number of threads and nodes in ICM (3 recommended)

typedef struct{
    volatile uint64_t *source;
    volatile uint64_t *sink;
    int pause;
    int run;
    pthread_mutex_t mutx;
    pthread_t thr;
} icm_thread_t;

typedef struct{
    volatile uint64_t nodes[_ICM_MAX_THREADS];
    icm_thread_t threads[_ICM_MAX_THREADS];
} icm_state_t;

void icm_init(icm_state_t* state);
void icm_fill64(icm_state_t* state, uint64_t* buffer, uint64_t count);
void icm_mix64(icm_state_t* state, uint64_t* buffer, uint64_t count);
void icm_stop(icm_state_t* icm);

#endif