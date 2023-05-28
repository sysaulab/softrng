/**
* @file stat.h
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Various Stats Library header */

#ifndef ___common_h
#define ___common_h

double ftime(void);

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

typedef struct {
    uint8_t bits_cache[0x10000];
    uint8_t bits_cache_ready;
    uint64_t count;
    uint64_t unique;
    uint64_t data[0x100000000/64];
    double score_unique;
} distribution;

typedef struct {
    double min;
    double sum;
    double max;
    uint64_t count;
} fstat_t;

void fstat_init(fstat_t* fstat);
void fstat_analyze(fstat_t* fstat, double value);
double fstat_min(fstat_t* fstat);
double fstat_max(fstat_t* fstat);
double fstat_avg(fstat_t* fstat);

void distribution_init( distribution* dist );
double deviation(double a, double b);
double update_score(distribution* dist);
void add_u64(distribution* dist, uint64_t value);
void init_bits(uint8_t* bits);

#endif