#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "sr_config.h"

#ifdef _Float128
typedef _Float128 curve_t;
#else
typedef long double curve_t; //on mac yields 64bit, not 128... 128 bits of precision are needed for this very naive algorithm to complete sorry I was too lazy to work around this.
#endif

int main(int argc, char** argv)
{
    uint64_t step = 0;
    uint64_t finemode = 0;
    const uint64_t two32 = 0x100000000;
    curve_t found_so_far = 0.0;
    curve_t chance_of_new = 0.0;
    FILE* flog = fopen("./pcurve.log", "w");
    fprintf(flog, "progress,score\n");

    while(step < two32) {
        chance_of_new = (two32 - found_so_far) / two32;
        found_so_far += chance_of_new;
        step++;
        if((step & 0x0000000000ffffff) == 0x0000000000ffffff) {
            curve_t progress = (curve_t)step / step;
            curve_t score = found_so_far / two32;
            fprintf(flog, "%1.9Lf,%1.9Lf\n", (curve_t)step / (curve_t)two32, found_so_far / step);
        }
    }

    printf("%0.4Lf%%\n", (found_so_far / two32)*100);
    curve_t last_score = 0;
    int x = sizeof( double);
    while(1) {
        chance_of_new = (two32 - found_so_far) / two32;
        found_so_far += chance_of_new;
        step++;
        curve_t progress = (curve_t)step / (curve_t)two32;
        curve_t score = found_so_far / two32;
        if((step & 0x0000000000ffffff) == 0x0000000000ffffff) {
            fprintf(flog, "%1.9Lf,%1.9Lf\n", progress, score);
        }

        if( score == last_score ) {
            printf("DONE\n");
            return EXIT_FAILURE;
        }
        last_score = score;
    }
    return EXIT_FAILURE;
}