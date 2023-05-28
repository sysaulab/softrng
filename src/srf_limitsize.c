/**
* @file keep.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Keep filter, relay at most n bytes from stdin to stdout */

#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

/**
* @brief Parse a string containing a size
* @author Sylvain Saucier
* @param in
* @param out
* Parse strings such as "15kb" or "1.375TB" and return byte value as double 
*/
double parseSize(char* text){
    double answer = atof(text);
    for(int x = 0; text[x] ; x++){
        switch (text[x]){
            case 'k': return answer * 1000.0;
            case 'm': return answer * 1000000.0;
            case 'g': return answer * 1000000000.0;
            case 't': return answer * 1000000000000.0;
            case 'p': return answer * 1000000000000000.0;
            case 'e': return answer * 1000000000000000000.0;
            case 'z': return answer * 1000000000000000000000.0;
            case 'y': return answer * 1000000000000000000000000.0;
            case 'K': return answer * 1024.0;
            case 'M': return answer * 1024.0 * 1024.0;
            case 'G': return answer * 1024.0 * 1024.0 * 1024.0;
            case 'T': return answer * 1024.0 * 1024.0 * 1024.0 * 1024.0;
            case 'P': return answer * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0;
            case 'E': return answer * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0;
            case 'Z': return answer * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0;
            case 'Y': return answer * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0 * 1024.0;}}
    return answer;}

int main(int argc, char** argv){
    if(argc < 2){
        printf("Distributed under Affero GNU Public Licence version 3\n\n");
        printf("usage : stdin | trim {size} | stdout\n");
        printf("    size is a fractional number with a suffix\n");
        printf("    metric suffix are lower case : k, m, g, t, p, e, z, y\n");
        printf("    base 2 suffix are upper case : K, M, G, T, P, E, Z, Y\n");
        printf("    Ex.: 1.44mb, 1GiB 0.001tb\n");
        return EXIT_FAILURE;}
    uint64_t bytes_requested = parseSize(argv[1]);
    uint64_t bytes_written = 0;
    while(1){
        uint8_t buffer[_SSRNG_BUFSIZE];
        uint64_t remains = (bytes_requested - bytes_written);
        if(!remains) break;
        uint64_t have = fread(&buffer, sizeof(uint8_t), _SSRNG_BUFSIZE, stdin);
        if(!have) break;
        bytes_written += fwrite(&buffer, sizeof(uint8_t), have > remains ? remains : have, stdout);}
    return EXIT_SUCCESS;
}