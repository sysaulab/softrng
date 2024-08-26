#!/bin/sh

echo "0000000000000000 == \c"
s-ent-password | f-limit 8 | f-hex
echo 

echo "f379180e00000000 == \c"
s-ent-password "pass" | f-limit 8 | f-hex
echo

echo "64f9fb3e9f87e100 == \c"
s-ent-password "password" | f-limit 8 | f-file ./test.bin | f-hex
echo

echo "s-file ./test.bin | srf_hex"
s-file ./test.bin | f-hex
echo
echo

echo "srs_fread ./test.bin | srf_xor "srs_fread ./test.bin" | srf_hex"
srs_fread ./test.bin | srf_xor "srs_fread ./test.bin" | srf_hex
echo
echo

echo "srs_zeros | srf_limit 10 | srf_hex"
srs_zeros | srf_limit 10 | srf_hex
echo
echo

echo "srs_getent | srf_limit 10 | srf_hex"
srs_getent | srf_limit 10 | srf_hex
echo
echo

echo "s-system | srf_limit 10 | srf_hex"
s-system | srf_limit 10 | srf_hex
echo
echo

echo "s-system | f-roxo64 | srf_limit 10 | srf_hex"
s-system | f-roxo64 | srf_limit 10 | srf_hex
echo
echo

echo "s-system | f-qxo64 | srf_limit 10 | srf_hex"
s-system | f-qxo64 | srf_limit 20 | srf_hex
echo
echo

rm ./test.bin

echo "srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole"
srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole
echo
echo

echo "sr_timertest"
sr_timertest