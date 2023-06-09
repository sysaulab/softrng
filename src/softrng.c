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

#include <stdio.h>
#include <unistd.h>
#include <string.h>
#include <stdlib.h>
#include <dirent.h>
#include <sys/stat.h>
#include "sr_default.h"

char dir_softrand[] = "/etc/softrng/";
char dir_bin[] = "/usr/local/bin/";
char dir_help[] = "/etc/softrng/help/";
char dir_modules[] = "/etc/softrng/modules/";

int file_exist(char* path)
{
    if(access(path, F_OK) == 0) return 1;
    else return 0;
}

int dir_exist(char* path)
{
    DIR* d = opendir(path);
    if (d)
    {
        return 1;
        closedir(d);
    }
    return 0;
}

int cmd_exist(char* c)
{
    char string_buf[1024] = "";
    strcpy(string_buf, "command -v ");
    strncat(string_buf, c, 1000);

    FILE* command_output = popen(string_buf, "r");
    fread(string_buf, sizeof(uint8_t), 1023, command_output);
    pclose(command_output);

    if( string_buf[0] == '/' )
    {
        return 1;
    }
    return 0;
}

void manual()
{
    char command[256] = "less<";
    strcat(command, dir_softrand);
    strcat(command, "manual.txt");
    system(command);
}

int alias(char* alias_name, char* alias_command)
{
    char cmd[201];
    char local_bin[200] = "";
    strncpy(local_bin, dir_bin, 200);
    strncat(local_bin, alias_name, 200);
    
    FILE* alias_file = fopen(local_bin, "w");
    if(!alias_file) return 0;

    fprintf(alias_file, "#!/bin/sh\n%s $@", alias_command);

    fclose(alias_file);
    strncpy(cmd, "sudo chmod +x ", 200);
    strncat(cmd, local_bin, 200);
    system(cmd);
    return 1;
}

int rem(char* a)
{
    char cmd[201];
    strncpy(cmd, dir_bin, 200);
    strncat(cmd, a, 200);
    return remove(cmd);
}

int scan_one_db(char* target_cmd, FILE* db_file)
{
    if(!db_file)
    {
        printf("Database file not open.\n");
        return 0;
    }
    int errors = 0;
    size_t len = 4096;
    char* line = malloc(len + 1);
    if(!line) return 2;
    char* short_name = calloc(len + 1, sizeof(char)); if(!short_name) exit(EXIT_FAILURE);
    char* short_cmd  = calloc(len + 1, sizeof(char)); if(!short_cmd)  exit(EXIT_FAILURE);
    if(!cmd_exist(target_cmd))
    {
        free(short_name);
        free(short_cmd);
        printf("Missing: %s\n\n", target_cmd);
        return 0;
    }
    else 
    {
        printf("Found: %s\n", target_cmd);
        while((getline(&line, &len, db_file)) != -1)
        {
            fscanf(db_file, "%s %4096[^\n]", short_name, short_cmd);

            if(!alias(short_name, short_cmd))
            {
                printf("cannot create: %s\n", short_name);
                errors++;
            }
            else
            {
                printf("created: %s\n", short_name);
            }
        }
        printf("\n");
    }

    if(errors)
    {
        printf("%i errors occured\n", errors);
        printf("Retry using administrator privileges (sudo softrng install).\n");
    }

    free(short_name);
    free(short_cmd);
    free(line);
    fclose(db_file);
    return 0;
}

void autoscan(char* path)
{
    DIR* d = opendir(path);
    struct dirent *e;

    if(d != NULL)
    {
        while((e = readdir(d)))
        {
            if(e->d_name[0] == '.')
                continue;
            char filename[1025] = "";
            strncpy(filename, path, 1024);
            strncat(filename, e->d_name, 1024);
            FILE* file = fopen(filename, "r");
            
            scan_one_db(e->d_name, file);
            fclose(file);
        }
    }
}

int mkfile(char* path, char* content)
{
    FILE* file = fopen(path, "w");
    if(file == NULL) return 1;
    fprintf(file, "%s", content);
    fclose(file);
    return 0;
}

void cfg_dir(char* path)
{
    if(!dir_exist(path))
    {
        if(mkdir(path, 0755))
        {
            printf("Error, cannot create folder: %s\n", path);
        }
    }
}

void cfg_file(char* path, char* content)
{
    char fullpath[1025];
    strncpy(fullpath, dir_softrand, 1024);
    strncat(fullpath, path, 1024);
    if(!file_exist(fullpath))
    {
        printf("%s\n", fullpath);
        if(mkfile(fullpath, content))
            printf("Error, cannot write to file: %s\n", fullpath);
    }
}

void force_cfg_file(char* path, char* content)
{
    char fullpath[1025];
    strncpy(fullpath, dir_softrand, 1024);
    strncat(fullpath, path, 1024);
    printf("%s\n", fullpath);
    if(mkfile(fullpath, content))
        printf("Error, cannot write to file: %s\n", fullpath);
}

void verify_config()
{
    cfg_dir(dir_softrand);
    cfg_dir(dir_help);
    cfg_dir(dir_modules);
//    cfg_dir(dir_bin);
    cfg_file("manual.txt", _files_manual);
    cfg_file("modules/softrng", _files_module_softrng);
    cfg_file("modules/dieharder", _files_module_dieharder);
    cfg_file("modules/RNG_test", _files_module_RNG_test);
    cfg_file("modules/RNG_output", _files_module_RNG_output);
}

void force_verify_config()
{
    cfg_dir(dir_softrand);
    cfg_dir(dir_help);
    cfg_dir(dir_modules);
//    cfg_dir(dir_bin);
    force_cfg_file("manual.txt", _files_manual);
    force_cfg_file("modules/softrng", _files_module_softrng);
    force_cfg_file("modules/dieharder", _files_module_dieharder);
    force_cfg_file("modules/RNG_test", _files_module_RNG_test);
    force_cfg_file("modules/RNG_output", _files_module_RNG_output);
}

int main(int argc, const char * argv[])
{
    for(int x = 1; x < argc; x++)
    {
        if(0 == strcmp("open", argv[x]))
        {
            system("open /etc/softrng");
            return EXIT_SUCCESS;
        }
        else if(0 == strcmp("setup", argv[x]))
        {
            verify_config();
            autoscan(dir_modules);
            return EXIT_SUCCESS;
        }
        else if(0 == strcmp("force", argv[x]))
        {
            force_verify_config();
            return EXIT_SUCCESS;
        }
        
        else if(0 == strcmp("manual", argv[x]))
        {
            manual();
            return EXIT_SUCCESS;
        }
    }

    printf("usage: sudo softrng setup     Install configuration and commands. Run as root.\n");
    printf("       softrng open      Open the configuration folder.\n");
    printf("       softrng manual      Open the configuration folder.\n");

    return EXIT_FAILURE;
}

