#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <pthread.h>
#include <math.h>
#include <unistd.h>

#include "sr_config.h"

int max_stage = 0;
int stage = 0; 
FILE* logfile;
FILE* resultfile;
double start;
double stage_start;

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
double ftime(void);
void distribution_init( distribution* dist );
double deviation(double a, double b);
double update_score(distribution* dist);
void add_u64(distribution* dist, uint64_t value);
void init_bits(uint8_t* bits);

void fstat_init(fstat_t* fstat)
{
    fstat->min = 0xffffffffffffffff;
    fstat->sum = 0;
    fstat->max = 0;
}

void fstat_analyze(fstat_t* fstat, double value)
{
    if( value < fstat->min )
        fstat->min = value;

    if( value > fstat->max )
        fstat->max = value;

    fstat->sum += value;
    fstat->count++;
}

double fstat_min(fstat_t* fstat)
{
    return fstat->min;
}

double fstat_max(fstat_t* fstat)
{
    return fstat->max;
}

double fstat_avg(fstat_t* fstat)
{
    return fstat->sum / fstat->count;
}

uint64_t count_bits_fast(uint16_t value, uint8_t* bits)
{
    return bits[value];
}

uint8_t count_bits_slow(uint16_t value)
{
    uint8_t answer = 0;
    while( value != 0 )
    {
        if( value & 1 )
            answer++;
        value = value >> 1;
    }
    return answer;
}

void init_bits(uint8_t* bits)
{
    int bit = 0;
    while ( bit < 0x10000 )
    {
        bits[bit] = count_bits_slow(bit);
        bit++;
    }
}

void count_u32_bits( distribution* dist )
{
    uint8_t* cache = dist->bits_cache;
    if(dist->bits_cache_ready == 0)
    {
        init_bits(cache);
    }

    uint64_t unique = 0;
    uint64_t segment = 0;
    while( segment < 0x100000000/64 )
    {
        uint16_t* value = (uint16_t*)&dist->data[segment];
        unique += (cache[value[0]] + cache[value[1]]) + (cache[value[2]] + cache[value[3]]);
        segment++;
    }
    dist->unique = unique;
}

double deviation( double a, double b )
{
    if (a < b) return a / b;
    return b / a;
}

void u32_update_score(distribution* dist)
{
    double target_unique = dist->count;

    if ( target_unique > 0x100000000 ) 
        target_unique = 0x100000000;
        
    count_u32_bits(dist);
    dist->score_unique = deviation(target_unique, dist->unique);
}

double update_score(distribution* dist)
{
    double score = 1.0;
    u32_update_score(dist);
    score *= dist->score_unique;
    return score;
}

void add_u32 (distribution* stat32, uint32_t value)
{
    uint64_t prefix = value / 64;
    uint64_t suffix = value % 64;
    stat32->data[prefix] |= (1LL << suffix);
    stat32->count++;
}

void add_u64( distribution* dist, uint64_t value )
{
    uint32_t* sub = (uint32_t*)&value;
    add_u32(dist, sub[0]);
    add_u32(dist, sub[1]);
}

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
