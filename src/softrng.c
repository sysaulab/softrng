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
    fprintf(alias_file, "#!/bin/sh\n%s $@", c);
    fclose(alias_file);
    strncpy(cmd, "sudo chmod +x ", 200);
    strncat(cmd, local_bin, 200);
    system(cmd);
    return 1;
}

int rem(char* a)
{
    char cmd[201];
    strncpy(cmd, "sudo rm ", 200);
    strncat(cmd, "/usr/local/bin/", 200);
    strncat(cmd, a, 200);
    return system(cmd);
}

int setup_keywords()
{
    uint64_t err = 0;
    if(!alias("HEX",  "srf_hex")) err++;
    if(!alias("INFO",  "srf_bench")) err++;
    if(!alias("MIXOR", "srf_multixor")) err++;
    if(!alias("PRNG", "srg_roxo64")) err++;
    if(!alias("RAND", "cat /dev/random")) err++;
    if(!alias("RNG", "SEED|PRNG")) err++;
    if(!alias("SEEDY", "srs_icm64mix")) err++;
    if(!alias("SEED", "srs_icm64")) err++;
    if(!alias("STOP", "srf_limitsize")) err++;
    if(!alias("ZERO", "cat /dev/zero")) err++;
    if(!alias("VOID", "srs_void")) err++;
    return 1;
}

int remove_keywords()
{
    uint64_t err = 0;
    if(!rem("HEX")) err++;
    if(!rem("INFO")) err++;
    if(!rem("MIXOR")) err++;
    if(!rem("PRNG")) err++;
    if(!rem("RAND")) err++;
    if(!rem("RNG")) err++;
    if(!rem("SEEDY")) err++;
    if(!rem("SEED")) err++;
    if(!rem("STOP")) err++;
    if(!rem("ZERO")) err++;
    if(!rem("VOID")) err++;
    return 1;
}

int main(int argc, const char * argv[])
{
    for(int x = 1; x < argc; x ++)
    {
        if(!strcmp(argv[x], "--install"))
        {
            if(setup_keywords())
            {
                fprintf(stderr, "Error occured, retry with admin privileges.");
                return EXIT_FAILURE;
            }
            return EXIT_SUCCESS;
        }
        else if(!strcmp(argv[x], "--remove"))
        {
            if(remove_keywords())
            {
                fprintf(stderr, "Error occured, retry with admin privileges.");
                return EXIT_FAILURE;
            }
            return EXIT_SUCCESS;
        }
    }
    manual();
    return EXIT_SUCCESS;
}
/* OLD STUFF DOWN THERE

















//CLASS FLAGS 
#define I 1 //requires input from source
#define O 2 //requires target tp receive output 
#define P 4 //prng generator, seed goes in sequence goes out
#define T 8 //target commands, where data goes to die, typical test or file...
#define S 16 //seed commands, my or may not require input.
#define F 32 //filter command, middle of crypto chain
#define E 128 //external command, depends on another package

int scan(item_list_t* list, char* db_path)
{
    FILE* db_file = fopen(db_path, "r");
    if(!db_file)
    {
        fprintf(stderr, "Error opening database file, please reinstall softrng.\n");
        return 0;
    }
    ssize_t read;
    uint64_t errors = 0;
    size_t len = 4096;
    char* line = malloc(len + 1);
    if(!line) exit(EXIT_FAILURE);
    printf("#COMMANDS");

    char* target_cmd = calloc(1025, sizeof(char)); if(!target_cmd) exit(EXIT_FAILURE);

    char* alias_name = calloc(1025, sizeof(char)); if(!alias_name) exit(EXIT_FAILURE);
    char* alias_class= calloc(1025, sizeof(char)); if(!alias_class) exit(EXIT_FAILURE);
    char* alias_cmd  = calloc(1025, sizeof(char)); if(!alias_cmd) exit(EXIT_FAILURE);
    char* alias_cmd_dirty  = calloc(1025, sizeof(char)); if(!alias_cmd) exit(EXIT_FAILURE);
    char* alias_desc = calloc(1025, sizeof(char)); if(!alias_desc) exit(EXIT_FAILURE);

    while ((read = getline(&line, &len, db_file)) != -1)
    {
        if(line[0] == ' ')
        {
            int x = 1;

            for(int start = x;line[x] != '\0';)
                if(line[x] != '|'){
                    alias_name[x-start] = line[x];
                    x++;}
                else{
                    alias_name[x-start] = '\0';
                    x++;
                    break;}

            for(int start = x;line[x] != '\0';)
                if(line[x] != '|'){
                    alias_class[x-start] = line[x];
                    x++;}
                else{
                    alias_name[x-start] = '\0';
                    x++;
                    break;}

            for(int start = x;line[x] != '\0';)
                if(line[x] != '|'){
                    alias_cmd_dirty[x-start] = line[x];
                    x++;}
                else{
                    alias_name[x-start] = '\0';
                    x++;
                    break;}

            for(int start = x;line[x] != '\0';)
                if(line[x] != '|'){
                    alias_desc[x-start] = line[x];
                    x++;}
                else{
                    alias_name[x-start] = '\0';
                    x++;
                    break;}

            escape_space(alias_cmd_dirty, alias_cmd);
            printf("  name %s\n", alias_name);
            printf("  class %s\n", alias_class);
            printf("  cmd %s\n", alias_cmd_dirty);
            printf("  cmdc %s\n", alias_cmd);
            printf(" %s\n", alias_name);
            if(!strncmp(target_cmd, alias_name, 1024))
            {
                soft_alias(alias_name, alias_cmd);
                if(save_config)
                    hard_alias(alias_name, alias_cmd);
            }
            add_item_to_list(class_from_str(alias_class), alias_name, alias_desc, alias_cmd, &db);
        }
        else
        {
            remove_newline(line);
            strncpy(target_cmd, line, 1024);
fprintf(stderr, "TARGET:%s\n", target_cmd);
            continue;
        }
    }
    printf("\n");
    if(errors > 0) fprintf(stderr, "To correct errors, reinstall softrng.\n");
    return !errors;
}







void remove_newline(char* str)
{
    int x = 0;
    int run = 1;
    while (run){
        switch (str[x]){
            case '\n':
            case '\r':
            case '\0':
                str[x] = '\0';
                run = 0;}
        x++;}
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




int main_menu()
{
    menu:
    printf("\nMain menu\n");
    printf("1. Change defaults (root)\n");
    printf("2. Help, read the manual\n");
    printf("3. View recognized commands\n");
    printf("q. Exit\n");
    printf("<enter>");
    fflush(stdout);

    while(1)
    {
        switch( getc(stdin) ) {
        case '1':
            if(!setup_keywords(0)) printf("Temporary shortcuts in effect for current session.\n");
            else printf("Permanent shortcuts saved to disk.\n");
            printf("<enter>");
            break;
        case '2':
            manual();
            printf("<enter>");
            break;
        case '3':
            printf("<enter>");
            break;
        case '4':
            break;
        case '5':
            break;

        case '9':
            break;
        case '\n':
        case '\r':
            printf("");
            break;
        case 'q':
        case 'Q':
        case '\e':
            return EXIT_SUCCESS;
        default :
            goto menu;
        }
    }
}








typedef uint16_t item_id_t;
typedef uint8_t item_class_t;

typedef struct {
    item_id_t id;
    item_class_t class;
    char* name;
    char* desc;
    char* cmd;
} item_t;

typedef struct {
    size_t capacity;
    size_t size;
    item_t* list;
} item_list_t;

item_list_t db;
item_id_t nextid = 0;
int save_config = 0; //requires root access

item_class_t class_from_str(char* str)
{
    item_class_t answer = 0;
    for(int x=0;!str[x];x++)
    {
        switch (str[x]) {
        case 'I': answer |= I; break;
        case 'O': answer |= O; break;
        case 'P': answer |= P; break;
        case 'T': answer |= T; break;
        case 'S': answer |= S; break;
        case 'F': answer |= F; break;
        case 'E': answer |= E; break;
        }
    }
    return answer;
}

char* dup_str(char* src)
{
    int l = strnlen(src, 1024);
    char* answer = malloc((1 + l) * sizeof(char));
    memcpy(answer, src, l);
    return answer;
}

item_id_t add_item_to_list(item_class_t class, char* name, char* desc, char* cmd, item_list_t* list)
{
    if(list->capacity < 1){
        list->capacity = 16;
        list->list = malloc(list->capacity * sizeof(item_t));
        if(list->list == NULL) exit(EXIT_FAILURE);}
    if(list->capacity - list->size < 1){
        list->capacity *= 2;
        list->list = realloc(list->list, list->capacity * sizeof(item_t));
        if(list->list == NULL) exit(EXIT_FAILURE);}
    list->list[list->size].id = nextid++;
    list->list[list->size].class = class;
    list->list[list->size].name = dup_str(name);
    list->list[list->size].cmd = dup_str(cmd);
    list->list[list->size].desc = dup_str(desc);
    list->size++;
    return nextid;
}


*/