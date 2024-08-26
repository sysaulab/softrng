#include <stdio.h>
#include <stdlib.h>
#include "sr_config.h"

int main(int argc, char** argv){
    char buffer[_SSRNG_BUFSIZE];
    while(fread(buffer, sizeof(char), _SSRNG_BUFSIZE, stdin));
    return EXIT_SUCCESS;
}
