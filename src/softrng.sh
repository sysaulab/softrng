#!/bin/sh

alias READ="< "
alias WRITE="> "
alias PASS="| "

alias LIMIT="srf_limitsize "
alias HEX="srf_hex "
alias MIXOR="srf_multixor" 
alias ZERO="cat /dev/zero "
alias RAND="cat /dev/random "

alias SEEDLESS="srs_icm64 "
alias SEED="MIXOR SEEDLESS RAND "
alias PRNG="srg_roxo64 "
alias RNG="SEED|PRNG "
