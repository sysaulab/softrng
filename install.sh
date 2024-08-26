#!/bin/sh
mkdir -p bin
mkdir -p logs

echo "Building SoftRNG"
echo "SOFTRNG INSTALL LOG (stdout)" > logs/softrng.std.txt
echo "SOFTRNG ERROR LOG (stderr)" > logs/softrng.err.txt
cc src/sr_timertest.c   -O2 -o ./bin/sr_timertest >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srs_password.c   -O2 -o ./bin/s-ent-password >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_bench.c      -O2 -o ./bin/f-peek >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srs_fread.c      -O2 -o ./bin/s-file >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_fwrite.c     -O2 -o ./bin/f-file >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_twrite.c     -O2 -o ./bin/t-file >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_hex.c        -O2 -o ./bin/f-hex >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_limit.c      -O2 -o ./bin/f-limit >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srt_hole.c       -O2 -o ./bin/t-hole >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_xor.c        -O2 -o ./bin/f-merge >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srf_fork.c       -O2 -o ./bin/f-fork >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srg_qxo64.c      -O2 -o ./bin/f-qxo64 >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srg_roxo64.c     -O2 -o ./bin/f-roxo64 >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srs_zeros.c      -O2 -o ./bin/s-zeros >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srs_getent.c     -O2 -o ./bin/s-getent >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srs_icm64.c      -O2 -o ./bin/s-chaos >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srt_bspec32.c    -O2 -o ./bin/t-bspec32 >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/srt_rngoutput.c  -O2 -o ./bin/sr_rngoutputshim >> logs/softrng.std.txt 2>> logs/softrng.err.txt
cc src/softrng.c        -O2 -o ./bin/softrng >> logs/softrng.std.txt 2>> logs/softrng.err.txt



echo "Building PractRand"
echo "PRACTRAND INSTALL LOG (stdout)" > logs/practrand.std.txt
echo "PRACTRAND ERROR LOG (stderr)" > logs/practrand.err.txt
unzip -u external/PractRand-master.zip >> logs/practrand.std.txt 2>> logs/practrand.err.txt
cd PractRand-master/unix >> logs/practrand.std.txt 2>> logs/practrand.err.txt
make >> ../../logs/practrand.std.txt 2>> ../../logs/practrand.err.txt
cd ../.. >> ../../logs/practrand.std.txt 2>> ../../logs/practrand.err.txt
cp PractRand-master/bin/RNG_* ./bin >> logs/practrand.std.txt 2>> logs/practrand.err.txt



echo "Building TestU01"
echo "TESTU01 INSTALL LOG (stdout)" > logs/testu01.std.txt
echo "TESTU01 ERROR LOG (stderr)" > logs/testu01.err.txt
unzip -u external/TestU01.zip >> logs/testu01.std.txt 2>> logs/testu01.err.txt
cd TestU01-1.2.3 >> logs/testu01.std.txt 2>> logs/testu01.err.txt
./configure --prefix="/usr/local" >> ../logs/testu01.std.txt 2>> ../logs/testu01.err.txt
make -j 6 >> ../logs/testu01.std.txt 2>> ../logs/testu01.err.txt
make -j 6 install >> ../logs/testu01.std.txt 2>> ../logs/testu01.err.txt
cd .. >> ../logs/testu01.std.txt 2>> ../logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-smallcrush src/sr_testu01small.c >> logs/testu01.std.txt 2>> logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-crush      src/sr_testu01.c >> logs/testu01.std.txt 2>> logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-bigcrush   src/sr_testu01big.c >> logs/testu01.std.txt 2>> logs/testu01.err.txt



cp -f bin/* /usr/local/bin/
cp -Rf softrng /etc
rm -Rf PractRand-master bin dieharder-master TestU01-1.2.3
chmod -Rf 777 logs
softrng install
echo "If you see Cleaning before the name of module, it has not been found."
echo "You can look into the logs folder to help you troubelshoot the problem."
echo ""
echo "To file a bug report : https://github.com/sysaulab/softrng/issues"
echo "Any other assistance : https://github.com/sysaulab/softrng/discussions"
echo

