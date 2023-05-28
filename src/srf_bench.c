/**
* @file bench.c
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
* @brief Live Display Thread
* @author Sylvain Saucier
* @param unused is unused
* This function is to be called as in independant thread.
* It display information about piped data.
*/
void* live(void* unused)
{
    while(1)
    {
        usleep(1000000/_SSRNG_FPS);
        fprintf(stderr, "\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b\b");
        double speed = bytes_written / (ftime() - start);
        if(speed > 1000000000000)   fprintf(stderr, "%0.3f tb/s (%llu)   ", speed / 1000000000000, bytes_written);
        else if(speed > 1000000000) fprintf(stderr, "%0.3f gb/s (%llu)   ", speed / 1000000000, bytes_written);
        else if(speed > 1000000)    fprintf(stderr, "%0.2f mb/s (%llu)   ", speed / 1000000, bytes_written);
        else if(speed > 1000)       fprintf(stderr, "%0.1f kb/s (%llu)   ", speed / 1000, bytes_written);
        else                        fprintf(stderr, "%0.0f b/s (%llu)    ", speed, bytes_written);
    }
}

/**
* @brief Main Thread
* @author Sylvain Saucier
* Main thread.
*/
int main(int argc, char** argv)
{
    uint64_t bytes_readed = 0;
    char buffer[_SSRNG_BUFSIZE];
    start = ftime();
    pthread_t thr;
    pthread_create(&thr, NULL, &live, NULL);
    while(1)
    {   
        uint64_t buf_received = fread(&buffer, sizeof(char), _SSRNG_BUFSIZE, stdin);
        if(!buf_received) break;
        bytes_readed += buf_received;
        bytes_written += fwrite(&buffer, sizeof(char), buf_received, stdout);
    }
    return EXIT_SUCCESS;
}