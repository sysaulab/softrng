/**
* @file libicm.c
* @author Sylvain Saucier <sylvain@sysau.com>
* @version 0.4.0
* @section LICENSE *
* This program is free software; you can redistribute it and/or
* modify it under the terms of the Affero GNU Public Licence version 3.
* Other licences available upon request.
* @section DESCRIPTION *
* Implementation of ICM library */

#include <pthread.h>
#include <stdint.h>

#include "libicm.h"
#include "sr_config.h"

/**
* @brief ICM Permutation Algorithm
* @author Sylvain Saucier
* @param in
* @param out
* This function contains the processing algorithm
* run by the icm threads
*/
int ___libicm_modify( volatile uint64_t* in, volatile uint64_t* out )
{
    if(in == NULL || out == NULL)
        return 1;
        
    uint64_t acc = 1;                                   /* newvalue : 128 unique prime numbers will be multiplied together in this accumulator */
    uint64_t primes[128] =                              /* primes : list of prime numbers, 256 first prime numbers. */
    #include "primes.large.txt"
    for( int x = 0; x < 64; x++ )
    {
        *in = (*in << 13) | (*in >> (64 - 13));         /* source is rotated by 13 bits... */
        acc = (acc << x) | (acc >> (64 - x));           /* smooth out bit distribution in acc */
        acc *= primes[(2 * x)+(1 & *in)];               /* accumulate a unique prime for this run... */
        *out ^= acc;                                    /* add uncertainty in the sink too as we go... */
    }
    //*out ^= acc;                                      /* finally XOR the accumulated product to the sink */
    return 0;
}

/**
* @brief stop the ICM chaos engine
* @author Sylvain Saucier
* @param icm Pointer to an ICM struct
* This function pause all the ICM threads
*/
void __icm_pause(icm_state_t* icm){
    for(int x = 0; x < _ICM_MAX_THREADS; x++){
        pthread_mutex_lock(&(icm->threads[x].mutx));
        icm->threads[x].pause = 1;}}


/**
* @brief start the ICM chaos engine
* @author Sylvain Saucier
* @param icm Pointer to an ICM struct
* This function pause all the ICM threads
*/
void __icm_start(icm_state_t* icm){
    for(int x = 0; x < _ICM_MAX_THREADS; x++){
        pthread_mutex_unlock(&(icm->threads[x].mutx));}}

/**
* @brief ICM Thread Function
* @author Sylvain Saucier
* @param raw void* pointer to an icm_thread_t
* This function pause all the ICM threads
*/
void* ___libicm_threadwork(void* raw)
{
    icm_thread_t* state = (icm_thread_t*) raw;
    while(1)
    {
        ___libicm_modify(state->source, state->sink);
        if(state->pause)
        {
            if(!state->run) pthread_exit(NULL);
            state->pause = 0;
            pthread_mutex_lock(&state->mutx);               //waiting for restart
            pthread_mutex_unlock(&state->mutx);             //unlocking the mutex right away...
            }
    }
    return NULL;
}

/**
* @brief ICM Initialization function
* @author Sylvain Saucier
* @param raw void* pointer to an icm_thread_t
* This function must be called before calling icm_fill()
*/
void icm_init(icm_state_t* state)
{
    for( int i = 0; i < _ICM_MAX_THREADS; i++ ){
        state->nodes[i] = 0;
        state->threads[i].source = &(state->nodes[i]);
        state->threads[i].sink = &(state->nodes[(i + 1) % _ICM_MAX_THREADS]);
        state->threads[i].run = 1;
        state->threads[i].pause = 1;
        pthread_mutex_init(&(state->threads[i].mutx), NULL);
        pthread_mutex_lock(&(state->threads[i].mutx));
        pthread_create(&(state->threads[i].thr), NULL, &___libicm_threadwork, &(state->threads[i]));}
    __icm_start(state);
    usleep(_ICM_WARMUP);
    __icm_pause(state);}

/**
* @brief ICM Stop function, stops all threads
* @author Sylvain Saucier
* @param raw void* pointer to an icm_thread_t
* This function can be called to quit the treads gracefully.
*/
void icm_stop(icm_state_t* state)
{
    for(int x = 0; x < _ICM_MAX_THREADS; x++) state->threads[x].run = 0;
    __icm_pause(state);
    for(int x = 0; x < _ICM_MAX_THREADS; x++) pthread_join(state->threads[x].thr, NULL);
}

/**
* @brief ICM Fill Function
* @author Sylvain Saucier
* @param state icm_state_t* pointer
* @param buffer uint64_t* pointer to a buffer to receive numbers
* @param count uint64_t how many uint64_t to wriye
* This function fills n bytes in a buffer
*/
void icm_fill64(icm_state_t* state, uint64_t* buffer, uint64_t count){
    __icm_start(state);
    uint64_t answer = 0;
    for( int i = 0; i < count; i++ ){
        usleep(_ICM_WAIT);
        for( int y = 0; y < _ICM_MAX_THREADS; y++ )
            answer ^= state->nodes[y];
        buffer[i] = answer;}
    __icm_pause(state);}

/**
* @brief ICM XOR Mix Function
* @author Sylvain Saucier
* @param raw void* pointer to an icm_thread_t
* Like icm_fill but with ^= instead of =.
*/
void icm_mix64(icm_state_t* state, uint64_t* destination, uint64_t count){
    __icm_start(state);
    uint64_t answer = 0;
    for( int i = 0; i < count; i++ ){
        usleep(_ICM_WAIT);
        for( int y = 0; y < _ICM_MAX_THREADS; y++ )
            answer = state->nodes[y];
        destination[i] ^= answer;}
    __icm_pause(state);}
