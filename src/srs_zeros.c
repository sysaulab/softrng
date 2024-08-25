#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv) {
    uint64_t zero = 0;
    while(1) fwrite(&zero, sizeof(zero), 1, stdout);
    return EXIT_SUCCESS;
}