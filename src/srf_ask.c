/**
* @file hex.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Pipe filter : stdin (binary) to stdout (hex) */
#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>
#include "sr_config.h"

#define buf_size 256

int main(int argc, char** argv)
{
    char buffer[buf_size];
    fprintf(stderr, "Please type something : ");

    fgets(buffer, buf_size, stdin);
    int len = strnlen(buffer, buf_size) - 1;
    int written = fwrite(buffer, sizeof(char), len, stdout);
    if(len != written)
    {
        fprintf(stderr, "len=%i, written=%i", len, written);
        exit(EXIT_FAILURE);
    }
    return EXIT_SUCCESS;
}
