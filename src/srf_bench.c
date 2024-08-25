#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include "sr_config.h"

#include <time.h>
double ftime(){
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

int main(int argc, char** argv)
{
    uint64_t bytes_written = 0;
    uint64_t bytes_readed = 0;
    char* buffer[_SSRNG_BUFSIZE];
    double start = -1;
    double last_update = ftime();
    int virgin = 1;
    while(1)
    {   
        uint64_t buf_received = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin);
        if(start < 0) {
            start = ftime();
        } else {
            if (ftime() > last_update + (1.0/_SSRNG_FPS) ) {
                fprintf(stderr, "\r");
                double speed = bytes_written / (ftime() - start);
                if     (speed > 1000000000000)  fprintf(stderr, "[%016llx] %0.3f tb/s (%llu)   ", *(uint64_t*)buffer, speed / 1000000000000, bytes_written);
                else if(speed > 1000000000)     fprintf(stderr, "[%016llx] %0.3f gb/s (%llu)   ", *(uint64_t*)buffer, speed / 1000000000, bytes_written);
                else if(speed > 1000000)        fprintf(stderr, "[%016llx] %0.2f mb/s (%llu)   ", *(uint64_t*)buffer, speed / 1000000, bytes_written);
                else if(speed > 1000)           fprintf(stderr, "[%016llx] %0.1f kb/s (%llu)   ", *(uint64_t*)buffer, speed / 1000, bytes_written);
                else                            fprintf(stderr, "[%016llx] %0.0f b/s (%llu)    ", *(uint64_t*)buffer, speed, bytes_written);
                last_update = ftime();
            }
        }
        if(!buf_received) break;
        bytes_readed += buf_received;
        bytes_written += fwrite(buffer, sizeof(char), buf_received, stdout);
    }
    return EXIT_SUCCESS;
}