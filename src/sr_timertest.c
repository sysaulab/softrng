#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include "libicm.h"

#include <time.h>
double ftime(void){
#ifdef __APPLE__
    return (double)(clock_gettime_nsec_np(CLOCK_REALTIME)) / 1000000000.0;
#else
//    fprintf(stderr, "UNTESTED CODE RUNNING REF#072384");
    struct timespec _linux_ts;
    clock_gettime(CLOCK_REALTIME, &_linux_ts);
    double _linux_time = _linux_ts.tv_sec;
    _linux_time += _linux_ts.tv_nsec / 1000000000.0;
    return _linux_time;
#endif
}
#define now() (ftime())

typedef struct {
    double min;
    double sum;
    double max;
    double count;
} fstat_t;

double get_minimum_nanosleep_delay(void)
{
    struct timespec t;
    t.tv_sec = 0;
    t.tv_nsec = 1;
    double start = now();
    nanosleep(&t, NULL);
    return now() - start;
}

double get_minimum_usleep_delay(void)
{
    double start = now();
    usleep(_ICM_WAIT);
    return now() - start;
}

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
    fstat->count = fstat->count + 1;
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

int main(int argc, char** argv)
{
    printf("Timer resolution test.\n\n");
    fstat_t stats;

    fstat_init(&stats);
    for(int x = 0; x < 10000; x++)
        fstat_analyze(&stats, get_minimum_usleep_delay());

    printf("usleep(%i) min :    %1.9fµs\n", _ICM_WAIT, fstat_min(&stats) * 1000000);
    printf("usleep(%i) avg :    %1.9fµs\n", _ICM_WAIT, fstat_avg(&stats) * 1000000 );
    printf("usleep(%i) max :    %1.9fµs\n", _ICM_WAIT, fstat_max(&stats) * 1000000 );

    fstat_init(&stats);
    for(int x = 0; x < 10000; x++)
        fstat_analyze(&stats, get_minimum_nanosleep_delay());

    printf("nanosleep(1) min : %1.9fµs\n", fstat_min(&stats) * 1000000 );
    printf("nanosleep(1) avg : %1.9fµs\n", fstat_avg(&stats) * 1000000 );
    printf("nanosleep(1) max : %1.9fµs\n", fstat_max(&stats) * 1000000 );
    return 0;
}

