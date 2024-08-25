#!/bin/sh

echo "srs_password"
srs_password | srf_limit 8 | srf_hex
echo 
echo 

echo "srs_password "pass" | srf_limit 8 | srf_hex"
srs_password "pass" | srf_limit 8 | srf_hex
echo
echo

echo "srs_password "password" | srf_limit 8 | srf_fwrite ./test.bin | srf_hex"
srs_password "password" | srf_limit 8 | srf_fwrite ./test.bin | srf_hex
echo
echo

echo "srs_fread ./test.bin | srf_hex"
srs_fread ./test.bin | srf_hex
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









echo "srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole"
srs_icm64 | srf_bench | srf_limit 1000000 | srt_hole
echo
echo

echo "sr_timertest"
sr_timertest