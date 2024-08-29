#include <stdatomic.h>
#include <stdio.h>
#include <stdint.h>
#include <pthread.h>
#include <stdlib.h>
#include <unistd.h>
#include <time.h>
#include "sr_config.h"

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

#define getMask(n) (0xFFFFffffFFFFffff >> (64-n))

void makeNiceNumber(uint64_t num, char* str) {
    uint64_t ret;
    char* suffix;
    uint64_t main;
    if (num >= 1LLU << 60){
        suffix = " Eb";
        main = num >> 60;
        ret = ((num & getMask(60))/(1LLU << 60))*100;//to avoid overflow when measuring exabytes's remainder
    } else if(num >= 1LLU << 50){
        suffix = " Pb";
        main = num >> 50;
        ret = ((num & getMask(50))*1000)/(1LLU << 50);
    } else if(num >= 1LLU << 40){
        suffix = " Tb";
        main = num >> 40;
        ret = ((num & getMask(40))*1000)/(1LLU << 40);
    } else if(num >= 1LLU << 30){
        suffix = " Gb";
        main = num >> 30;
        ret = ((num & getMask(30))*100)/(1LLU << 30);
        sprintf(str, "%llu.%02llu%s", main, ret, suffix);
        return;
    } else if(num >= 1LLU << 20){
        suffix = " Mb";
        main = num >> 20;
        ret = ((num & getMask(20))*10)/(1LLU << 20);
        sprintf(str, "%llu.%01llu%s", main, ret, suffix);
        return;
    } else if(num >= 1LLU << 10){
        suffix = " Kb";
        main = num >> 10;
        sprintf(str, "%llu%s", main, suffix);
        return;
    } else {
        suffix = " b";
        main = num;
        sprintf(str, "%llu%s", main, suffix);
        return;
    }
    sprintf(str, "%llu.%03llu%s", main, ret, suffix);
}
int main(int argc, char** argv) {
    uint64_t bytes_written = 0;
    uint64_t bytes_readed = 0;
    char* buffer[_SSRNG_BUFSIZE];
    double start = -1;
    double last_update = ftime();
    int virgin = 1;
    while(1) {
        uint64_t buf_received = fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin);
        if(start < 0) {
            start = ftime();
        } else {
            if (ftime() > last_update + (1.0/_SSRNG_FPS) ) {
                uint64_t speed = bytes_written / (ftime() - start);
                char nice_speed[100] = "";
                char nice_total[100] = "";
                makeNiceNumber(speed, &nice_speed[0]);
                makeNiceNumber(bytes_written, &nice_total[0]);
                fprintf(stderr, "\r[%016llx] (%s) %s/sec   ", *(uint64_t*)buffer, nice_total, nice_speed);
                last_update = ftime();
            }
        }
        if(!buf_received) break;
        bytes_readed += buf_received;
        bytes_written += fwrite(buffer, sizeof(char), buf_received, stdout);
    }
    return EXIT_SUCCESS;
}
