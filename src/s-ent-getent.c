#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/random.h>
#include "sr_config.h"

int main(int argc, const char * argv[]) {
    char out[256];
    while(1) {
        int result = getentropy(&out, 256);
        fwrite(&out, sizeof(char), 256, stdout);
    }
    return EXIT_SUCCESS;
}
