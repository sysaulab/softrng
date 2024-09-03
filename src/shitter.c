#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <string.h>
#include <pthread.h>
#include <unistd.h>
#include <time.h>
#include "sr_config.h"
//Super conservative settings, focus on security not speed.
#define _ICM_WARMUP         100                    // Warmup after init in microseconds
#define _ICM_WAIT           5                      // Sleeping time in microseconds
#define _ICM_MAX_THREADS    3                       // number of threads and nodes in ICM (3 recommended)

typedef struct{
    volatile uint64_t *source;
    volatile uint64_t *sink;
    int pause;
    int run;
    pthread_mutex_t mutx;
    pthread_t thr;
} icm_thread_t;

typedef struct{
    volatile uint64_t nodes[_ICM_MAX_THREADS];
    icm_thread_t threads[_ICM_MAX_THREADS];
} icm_state_t;

void init(icm_state_t* state);
void fill(icm_state_t* state, uint64_t* buffer, uint64_t count);
void icm_mix64(icm_state_t* state, uint64_t* buffer, uint64_t count);
void stop(icm_state_t* icm);

int ___libicm_modify( volatile uint64_t* in, volatile uint64_t* out ){
    if(in == NULL || out == NULL)
        return 1;
    uint64_t acc = 1;                                   /* newvalue : 128 unique prime numbers will be multiplied together in this accumulator */
    uint64_t primes[128] = {2985113428254948959ULL,2133371973447287789ULL,9566925003104080411ULL,6083414080430737261ULL,3191658566037706763ULL,6913367267901031997ULL,3051762490125913007ULL,4496279044463191703ULL,2879325911889076339ULL,1685450054146532617ULL,3665492224534333199ULL,5457405118009752311ULL,9603674174905975319ULL,5115570615370213399ULL,2085406057152991381ULL,3944123521248288751ULL,7068318273059084621ULL,9757209530458778617ULL,4605875306346355289ULL,6173263078309968247ULL,7762991333463966857ULL,9020426977113888977ULL,7479023155365493211ULL,1106308364738794853ULL,6407388572493574733ULL,2575377960084637447ULL,3570804859728957167ULL,9744215084471400349ULL,5786617550671126231ULL,3349473746601511027ULL,4095923646587284861ULL,9337574612835542873ULL,3860411135543820653ULL,6490253556739218821ULL,8764900392399904901ULL,3339372363887497583ULL,6577701767928311429ULL,9172672799009443051ULL,2788564470716716463ULL,7776210437768060567ULL,9071396938031495159ULL,5357072389526583433ULL,6633238451589839561ULL,6234713344370965483ULL,1756087432259377561ULL,3210111471103586737ULL,2408295824875360559ULL,8027800137025286071ULL,1345382896972678691ULL,9079595888451204607ULL,6556450002823963531ULL,7301373103704944939ULL,1213337252585919101ULL,8574942160927332881ULL,8566185364500201241ULL,6910045784078487233ULL,2298581249625702883ULL,2596136803677419831ULL,8990581036371057601ULL,3046889204803159003ULL,6864701819024214703ULL,9142639517683050829ULL,3257525145999184439ULL,5916350576516314903ULL,1991008946812235681ULL,5221054210441722691ULL,4658950185140640559ULL,7616154792906075869ULL,3789735489826889479ULL,6040491638497194979ULL,4531551599015040559ULL,8347022247974757701ULL,5867558032219360441ULL,4346307676337591453ULL,1905346606445378533ULL,7219759323757378847ULL,8561163821217102983ULL,6227762971465314193ULL,1778354250598502561ULL,5435722899178511549ULL,7859180386742822047ULL,3610133528213982527ULL,4577787558750808879ULL,3974497005134013109ULL,3241288686894948329ULL,3206546850542044487ULL,2621128575668534623ULL,6945339563322422683ULL,2086055779082649859ULL,9237096783908108657ULL,1916568609042647407ULL,8362781431934621017ULL,6122543372115730003ULL,6200399818195771541ULL,1259180073496439959ULL,4347405785030416513ULL,4385862544209720253ULL,5836562284002027449ULL,3121678407354608831ULL,4319136453194505203ULL,9085838656424873461ULL,5448516463713100871ULL,4520015808866767003ULL,2892328915328607301ULL,3897908110284010777ULL,6252018905800065119ULL,3278710975885668139ULL,2623408458547857527ULL,3055342605008846819ULL,5255646576194862401ULL,5611094629836787733ULL,3296996236992951511ULL,3283558997806116761ULL,2823396471863459119ULL,9572308265712226037ULL,8290430615658140401ULL,2621246478142939039ULL,1300291691995934101ULL,1143863624227711889ULL,6789671898597272039ULL,1593529243588418053ULL,6573565795099723367ULL,7154224095543629129ULL,2355509713379653547ULL,7691022839137400143ULL,2727794217173953619ULL,8541344794240933387ULL,6490328854741497349ULL};
    for( int x = 0; x < 64; x++ ) {
        *in = (*in << 13) | (*in >> (64 - 13));         /* source is rotated by 13 bits... */
        acc = (acc << x) | (acc >> (64 - x));           /* smooth out bit distribution in acc */
        acc *= primes[(2 * x)+(1 & *in)];               /* accumulate a unique prime for this run... */
        *out += acc ^ *in;                              /* add uncertainty in the sink too as we go... */
    }
    *out ^= acc;                                        /* finally XOR the accumulated product to the sink */
    return 0;
}

void __icm_pause(icm_state_t* icm){
    for(int x = 0; x < _ICM_MAX_THREADS; x++){
        pthread_mutex_lock(&(icm->threads[x].mutx));
        icm->threads[x].pause = 1;
    }
}

void __icm_start(icm_state_t* icm){
    for(int x = 0; x < _ICM_MAX_THREADS; x++){
        pthread_mutex_unlock(&(icm->threads[x].mutx));
    }
}

void* ___libicm_threadwork(void* raw){
    icm_thread_t* state = (icm_thread_t*) raw;
    while(1){
        ___libicm_modify(state->source, state->sink);
        if(state->pause){
            if(!state->run)
                pthread_exit(NULL);
            state->pause = 0;
            pthread_mutex_lock(&state->mutx);               //waiting for restart
            pthread_mutex_unlock(&state->mutx);             //unlocking the mutex right away...
        }
    }
    return NULL;
}

void init(icm_state_t* state){
    for( int i = 0; i < _ICM_MAX_THREADS; i++ ){
        state->nodes[i] = 0;
        state->threads[i].source = &(state->nodes[i]);
        state->threads[i].sink = &(state->nodes[(i + 1) % _ICM_MAX_THREADS]);
        state->threads[i].run = 1;
        state->threads[i].pause = 1;
        pthread_mutex_init(&(state->threads[i].mutx), NULL);
        pthread_mutex_lock(&(state->threads[i].mutx));
        pthread_create(&(state->threads[i].thr), NULL, &___libicm_threadwork, &(state->threads[i]));
    }
    __icm_start(state);
    usleep(_ICM_WARMUP);
    __icm_pause(state);
}

void fill(icm_state_t* state, uint64_t* buffer, uint64_t count) {
    __icm_start(state);
    uint64_t answer = 0;
    for( int i = 0; i < count; i++ )
    {
        answer = 0;
        usleep(_ICM_WAIT);
        for( int y = 0; y < _ICM_MAX_THREADS; y++ )
            answer ^= state->nodes[y];
        buffer[i] = answer;
    }
    __icm_pause(state);
}

int main(int argc, const char * argv[]) {
    uint64_t buffer[_SSRNG_BUFLEN];
    icm_state_t state;
    uint64_t pool[4][65536];
    union {
        uint64_t u64;
        uint16_t u16[4];
    } index;
    index.u64 = 0;
    icm_init(&state);
    fprintf(stderr, "▁");
    fill(&state, &pool[0][0], 65536);
    fprintf(stderr, "\b▃");
    icm_fill64(&state, &pool[1][0], 65536);
    fprintf(stderr, "\b▅");
    icm_fill64(&state, &pool[2][0], 65536);
    fprintf(stderr, "\b▇");
    icm_fill64(&state, &pool[3][0], 65536);
    fprintf(stderr, "\b█");
    fprintf(stderr, "\b");
    while(1) {
        for( uint64_t y = 0; y < _SSRNG_BUFLEN; y++ ) {
            index.u64 = index.u64 + 7776210437768060567ULL;// *MUST* be prime and close to 2^63
            buffer[y] = 
            pool[0][index.u16[0]] ^ 
            pool[1][index.u16[1]] ^ 
            pool[2][index.u16[2]] ^ 
            pool[3][index.u16[3]];
        }
        fwrite(buffer, sizeof(uint64_t), _SSRNG_BUFLEN, stdout);
    }
    return EXIT_SUCCESS;
}
