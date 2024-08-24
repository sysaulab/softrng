#!/bin/sh
mkdir release

cc src/sr_timertest.c   -O2 -o ./release/sr_timertest
cc src/srs_password.c   -O2 -o ./release/srs_password
cc src/srf_bench.c      -O2 -o ./release/srf_bench
cc src/srs_fread.c      -O2 -o ./release/srs_fread
cc src/srf_fwrite.c     -O2 -o ./release/srf_fwrite
cc src/srf_hex.c        -O2 -o ./release/srf_hex
cc src/srf_limit.c      -O2 -o ./release/srf_limit
cc src/srt_hole.c       -O2 -o ./release/srt_hole
cc src/srf_xor.c        -O2 -o ./release/srf_xor
cc src/srg_qxo64.c      -O2 -o ./release/srg_qxo64
cc src/srg_roxo64.c     -O2 -o ./release/srg_roxo64
cc src/srs_zeros.c      -O2 -o ./release/srs_zeros
cc src/srs_getent.c     -O2 -o ./release/srs_getent
cc src/srs_icm64.c      -O2 -o ./release/srs_icm64
cc src/srt_bspec32.c    -O2 -o ./release/srt_bspec32
cc src/softrng.c        -O2 -o ./release/softrng
