/**
* @file sleepinfo.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Program, display statistics on usleep() ans nanosleep() */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <unistd.h>
#include <string.h>
#include <time.h>
#include "common.h"
#include "common.c"//Single stage build... I know bad practice. Easier than maintaining autotools for small commands.

double get_minimum_nanosleep_delay()
{
    struct timespec t;
    t.tv_sec = 0;
    t.tv_nsec = 1;
    double start = ftime();
    nanosleep(&t, NULL);
    return ftime() - start;
}

double get_minimum_usleep_delay()
{
    double start = ftime();
    usleep(1);
    return ftime() - start;
}

int main(int argc, char** argv)
{
    printf("Timer resolution test.\n\n");
    fstat_t stats;

    fstat_init(&stats);
    for(int x = 0; x < 1000000; x++)
        fstat_analyze(&stats, get_minimum_usleep_delay());

    printf("usleep(1) min :    %1.1fµs\n", fstat_min(&stats) * 1000000);
    printf("usleep(1) avg :    %1.1fµs\n", fstat_avg(&stats) * 1000000);
    printf("usleep(1) max :    %1.1fµs\n", fstat_max(&stats) * 1000000);

    fstat_init(&stats);
    for(int x = 0; x < 100000; x++)
        fstat_analyze(&stats, get_minimum_nanosleep_delay());

    printf("nanosleep(1) min : %1.1fµs\n", fstat_min(&stats) * 1000000);
    printf("nanosleep(1) avg : %1.1fµs\n", fstat_avg(&stats) * 1000000);
    printf("nanosleep(1) max : %1.1fµs\n", fstat_max(&stats) * 1000000);
    return 0;
}