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

void manual()
{
    system("less</etc/softrng/manual.txt");
}

int alias(char* a, char* c)
{
    char cmd[201];
    char local_bin[200] = "/usr/local/bin/";
    strncat(local_bin, a, 200);
    FILE* alias_file = fopen(local_bin, "w");
    if(!alias_file) return 0;
    fprintf(alias_file, "#!/bin/sh\n%s $@ ", c);
    fclose(alias_file);
    strncpy(cmd, "sudo chmod +x ", 200);
    strncat(cmd, local_bin, 200);
    system(cmd);
    return 1;
}

int rem(char* a)
{
    char cmd[201];
    strncpy(cmd, "/usr/local/bin/", 200);
    strncat(cmd, a, 200);
    return remove(cmd);
}

int scan(char* target_cmd, FILE* db_file)
{
    if(!db_file) return 1;
    int errors = 0;
    size_t len = 4096;
    char* line = malloc(len + 1);
    if(!line) return 2;
    char* alias_name = calloc(len + 1, sizeof(char)); if(!alias_name) exit(EXIT_FAILURE);
    char* alias_cmd  = calloc(len + 1, sizeof(char)); if(!alias_cmd) exit(EXIT_FAILURE);
    if(!cmd_exist(target_cmd))
    {
        return 0;
    }
    else 
    {
        printf("%s: ", target_cmd);
        while ((getline(&line, &len, db_file)) != -1)
        {
            fscanf(db_file, "%s %s", alias_name, alias_cmd);
            if(!alias(alias_name, alias_cmd)) errors++;
            else printf("%s ", alias_name);
        }
        printf("\n");
    }

    if(errors)
    {
        printf("%i errors occured\n", errors);
        printf("Retry using administrator privileges (sudo softrng install).\n");
    }

    free(alias_name);
    free(alias_cmd);
    free(line);
    fclose(db_file);
    return 0;
}

int unscan(char* target_cmd, FILE* db_file)
{
    if(!db_file) return 1;
    int errors = 0;
    size_t len = 4096;
    char* line = malloc(len + 1);
    if(!line) return 2;
    char* alias_name = calloc(len + 1, sizeof(char)); if(!alias_name) exit(EXIT_FAILURE);
    char* alias_cmd  = calloc(len + 1, sizeof(char)); if(!alias_cmd) exit(EXIT_FAILURE);
    printf("%s: ", target_cmd);
    if(!cmd_exist(target_cmd))
    {
        printf(" NOT FOUND\n");
        return 0;
    }
    else 
    {
        while ((getline(&line, &len, db_file)) != -1)
        {
            fscanf(db_file, "%s %s", alias_name, alias_cmd);
            if(!rem(alias_name)) errors++;
            else printf("%s ", alias_name);
            fflush(stdout);
        }
        printf("\n");
    }

    if(errors)
    {
        printf("%i errors occured\n", errors);
        printf("Retry using administrator privileges (sudo softrng remove).\n");
    }

    free(alias_name);
    free(alias_cmd);
    free(line);
    fclose(db_file);
    return 0;
}

int main(int argc, const char * argv[])
{
    for(int x = 1; x < argc; x ++)
    {
        if(!strcmp(argv[x], "install"))
        {
            scan("softrng", fopen("/etc/softrng/modules/softrng.cmd.txt", "r"));
            scan("RNG_test", fopen("/etc/softrng/modules/RNG_test.cmd.txt", "r"));
            scan("RNG_output", fopen("/etc/softrng/modules/RNG_output.cmd.txt", "r"));
            scan("dieharder", fopen("/etc/softrng/modules/dieharder.cmd.txt", "r"));
            scan("missing", fopen("/etc/softrng/modules/missing.cmd.txt", "r"));
            return EXIT_SUCCESS;
        }

        if(!strcmp(argv[x], "remove"))
        {
            unscan("softrng", fopen("/etc/softrng/modules/softrng.cmd.txt", "r"));
            unscan("RNG_test", fopen("/etc/softrng/modules/RNG_test.cmd.txt", "r"));
            unscan("RNG_output", fopen("/etc/softrng/modules/RNG_output.cmd.txt", "r"));
            unscan("dieharder", fopen("/etc/softrng/modules/dieharder.cmd.txt", "r"));
            unscan("missing", fopen("/etc/softrng/modules/missing.cmd.txt", "r"));
            return EXIT_SUCCESS;
        }
    }
    manual();
    return EXIT_SUCCESS;
}

