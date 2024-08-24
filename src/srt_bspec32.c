/**
* @file srtest_bitspectrum32.h
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Simple test to verify if a strem contains all 32bit integers
* when observed 32 bits at a time. 
* Useful to test seed in reset mode and make nice graphs*/

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <math.h>
#include <unistd.h>

#include "common.h"
#include "common.c"
#include "sr_config.h"

int max_stage = 0;
int stage = 0; 
FILE* logfile;
FILE* resultfile;
double start;
double stage_start;

/**
* @brief Live Display Thread
* @author Sylvain Saucier
* @param raw is distribution* passed as void*
* This function is to be called as in independant thread.
* It display information about piped data.
*/
void* live_view(void* raw){
    distribution* dist = raw;
    while(1){
        sleep(1);
        update_score(dist);
        double ftime_elapsed = ftime() - start;
        double progress = (dist->count / (double)0x100000000);
        double iter_per_sec = progress / ftime_elapsed;
        double iter_remain = 1 - progress;
        double ftime_remaining = iter_remain / iter_per_sec;
        double kbps = (dist->count / ftime_elapsed)/1024;
        if( progress > 1 ) ftime_remaining = ftime() - start;
        uint32_t hours = floor(ftime_remaining / 3600);
        double f_remaining = ftime_remaining - (hours*3600);
        uint32_t mins = floor(f_remaining / 60);
        f_remaining = f_remaining - (mins*60);
        uint32_t secs = f_remaining;
        fprintf(stderr, "\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b");
        fprintf(stderr, "%.15f - %02uh%02um%02us - %.0fKiB/s - %u/%u", dist->score_unique, hours, mins, secs, kbps, stage, max_stage);
        fprintf(logfile, "%.12f,%.12f,%.12f\n", progress, dist->score_unique, ftime_elapsed );}
    return NULL;}

int main(int argc, const char * argv[]){
    double stop = 1.0;
    uint64_t* buffer;
    distribution* dist;
    pthread_t live;
    size_t readed = 1;
    buffer = calloc(_SSRNG_BUFLEN, sizeof(uint64_t));
    dist = calloc(1, sizeof(distribution));
    if( buffer == NULL || dist == NULL ){
        fprintf(stderr, "FATAL ERROR - cannot allocate memory buffer");
        return EXIT_FAILURE;}
    if(argc > 1) max_stage = atoi(argv[1]);
    if(max_stage < 0) max_stage = 0;
    if(max_stage > 10) max_stage = 10;
    logfile = fopen("bspec32.log", "a");
    resultfile = fopen("bspec32.txt", "a");
    fprintf(logfile, "progress,score,seconds\n");
    pthread_create(&live, NULL, &live_view, dist);
    start = ftime();
    while(1){
        readed = fread(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdin);
        if(!readed) break;

        for(int x = 0; x < readed; x++) 
            add_u64(dist, buffer[x]);

        switch (stage){
            case 0: //pre 1.0x
                if( (dist -> count / (double) 0x100000000) >= 1.0 ) {
                    update_score(dist);
                    printf("\nstage 0 : %0.9f\n", dist->score_unique);
                    fflush(stdout);
                    fprintf(resultfile, "stage 0 : %0.9f\n", dist->score_unique);
                    fflush(resultfile);
                    stage_start = ftime();
                    stage++;
                    if(stage > max_stage) {
                        goto leave;
                    }
                }
                break;
            case 1: stop = 0.9; goto result;
            case 2: stop = 0.99; goto result;
            case 3: stop = 0.999; goto result;
            case 4: stop = 0.9999; goto result;
            case 5: stop = 0.99999; goto result;
            case 6: stop = 0.999999; goto result;
            case 7: stop = 0.9999999; goto result;
            case 8: stop = 0.99999999; goto result;
            case 9: stop = 0.999999999; goto result;
            case 10:
                if( dist->unique == 0x100000000 ) {
                    printf("\nstage %u : %0.9f\n", stage, (double)(dist->count / (double)0x100000000));
                    fflush(stdout);
                    fprintf(resultfile, "stage %u : %0.9f\n", stage, (double)(dist->count / (double)0x100000000));
                    fflush(resultfile);
                    goto leave;
                }
                break;
            result:
                if( dist->score_unique > stop ) {
                    printf("\nstage %u : %0.9f\n", stage, (double)(dist->count / (double)0x100000000));
                    fflush(stdout);
                    fprintf(resultfile, "stage %u : %0.9f\n", stage, (double)(dist->count / (double)0x100000000));
                    fflush(resultfile);
                    stage_start = ftime();
                    stage++;
                    if(stage > max_stage)
                        goto leave;
                }
        }
    }
    leave:
    free(buffer);
    return EXIT_SUCCESS;
}