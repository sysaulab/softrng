#!/bin/sh

echo "0000000000000000 == \c"
s-ent-password | f-limit 8 | f-hex
echo 

echo "f379180e00000000 == \c"
s-ent-password "pass" | f-limit 8 | f-hex
echo

echo "64f9fb3e9f87e100 == \c"
s-ent-password "password" | f-limit 8 | f-file test.bin | f-hex
echo

echo "64f9fb3e9f87e100 == \c"
s-file test.bin | f-hex
echo

## f-merge is broken
echo "## f-merge is broken"
s-file test.bin | f-merge s-file test.bin | f-hex 
echo

echo "00000000000000000000 == \c"
s-zeros | f-limit 10 | f-hex
echo

srs_getent | srf_limit 10 | srf_hex
echo " != \c"
srs_getent | srf_limit 10 | srf_hex
echo

s-system | srf_limit 10 | srf_hex
echo " != \c"
s-system | srf_limit 10 | srf_hex
echo

s-system | f-roxo64 | srf_limit 10 | srf_hex
echo " != \c"
s-system | f-roxo64 | srf_limit 10 | srf_hex
echo

s-system | f-qxo64 | srf_limit 10 | srf_hex
echo " != \c"
s-system | f-qxo64 | srf_limit 10 | srf_hex
echo

 | srf_limit 10 | srf_hex
echo " != \c"
 | srf_limit 10 | srf_hex
echo


rm ./test.bin

echo "srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole"
srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole
echo
echo

echo "sr_timertest"
sr_timertest