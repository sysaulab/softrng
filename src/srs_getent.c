/**
* @file seedy.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Wrapper for getentropy() */

#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <sys/random.h>

int main(int argc, const char * argv[])
{
    char out[256];
    while(1)
    {
        getentropy(out, 256);
        fwrite(out, sizeof(char), 256, stdout);
    }
    return EXIT_SUCCESS;
}
