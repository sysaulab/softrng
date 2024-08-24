/**
* @file softrng.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Main program */

#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdlib.h>

#include "libicm.h"
#include "common.h"
#include "sr_config.h"

int hard_alias(char* a, char* c)
{
    char cmd[200];
    char local_bin[200] = "/usr/local/bin/";
    strncat(local_bin, a, 20);
    FILE* alias_file = fopen(local_bin, "w");
    if(!alias_file) return 0;
    fprintf(alias_file, "#!/bin/sh\n%s", c);
    fclose(alias_file);
    strncpy(cmd, "sudo chmod +x ", 20);
    strncat(cmd, local_bin, 20);
    system(cmd);
    return 1;
}

int soft_alias(char* a, char* c)
{
    char cmd[200];
    strncpy(cmd, "alias ", 7);
    strncat(cmd, a, 20);
    strncat(cmd, "=\"", 3);
    strncat(cmd, c, 20);
    strncat(cmd, "\"", 2);
    system(cmd);
    return 1;
}

int create_shortcut(char* a, char* c)
{
    if(!hard_alias(a, c))
        return soft_alias(a, c);
    return 0;
}

int cmd_exist(char* c){
    char cmd[1024] = "command -v ";
    strncat(cmd, c, 1000);
    FILE* out = popen(cmd, "r");
    fread(&cmd, 1023, sizeof(uint8_t), out);
    pclose(out);
    if( cmd[0] == '\0' ) return 0;
    if( cmd[0] == '/' ) return 1;
    return -1;
}

void escape_space(char* str, char* dest)
{
    int x, y = 0;
    while (str[x] != '\0')
    {
        if(str[x] == ' ')
            dest[y++] = '\\';
        dest[y++] = str[x++];
    }
    dest[y] = '\0';
}

int main(int argc, const char** argv){
    printf("rngsetup #VERSION 0.4.0 #LICENCE ");
    printf("This program is free software; you can redistribute it and/or modify it under the terms of the Affero GNU Public Licence version 3. ");
    int perm = 0;
    int seedless = 0;
    for (int x = 1; x < argc;x++ )
    {
        if(!strcmp(argv[x], "--permanent")) perm = 1;
        if(!strcmp(argv[x], "--seedless")) seedless = 1;
    }

    if(perm)
    {
        hard_alias("seed", "srs_icm64mix</dev/random");
        hard_alias("seedless", "srs_icm64");
        hard_alias("prng", "srg_roxo64-8kc");
        hard_alias("rng",  "seed|prng");
    }
    else
    {
        soft_alias("seed", "srs_icm64mix</dev/random");
        soft_alias("seedless", "srs_icm64mix</dev/random");
        soft_alias("prng", "srg_roxo64-8kc");
        soft_alias("rng",  "seed|prng");
    }

    return EXIT_SUCCESS;
}