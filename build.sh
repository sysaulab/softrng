#!/bin/sh
mkdir bin



echo
echo "Building SoftRNG"
echo

cc src/sr_timertest.c   -O2 -o ./bin/timertest
cc src/srs_password.c   -O2 -o ./bin/s-password
cc src/srf_bench.c      -O2 -o ./bin/f-peek
cc src/srs_fread.c      -O2 -o ./bin/s-file
cc src/srf_fwrite.c     -O2 -o ./bin/f-file
cc src/srf_hex.c        -O2 -o ./bin/f-hex
cc src/srf_limit.c      -O2 -o ./bin/f-limit
cc src/srt_hole.c       -O2 -o ./bin/t-hole
cc src/srf_xor.c        -O2 -o ./bin/f-xor
cc src/srg_qxo64.c      -O2 -o ./bin/f-qxo64
cc src/srg_roxo64.c     -O2 -o ./bin/f-roxo64
cc src/srs_zeros.c      -O2 -o ./bin/s-zeros
cc src/srs_getent.c     -O2 -o ./bin/s-getent
cc src/srs_icm64.c      -O2 -o ./bin/s-chaos
cc src/srt_bspec32.c    -O2 -o ./bin/t-bspec32
cc src/softrng.c        -O2 -o ./bin/softrng



echo
echo "Building PractRand"
echo
unzip -u external/PractRand-master.zip
cd PractRand-master/unix
make
cd ../..
cp PractRand-master/bin/RNG_* ./bin



echo
echo "Building TestU01"
echo
unzip -u external/TestU01.zip
cd TestU01-1.2.3
#Uncomment the end of the next line to install on Linux/aarch64.
#Adjust if configure complain about "unknown build type"
#We should look to make this more portable.
./configure --prefix="/usr/local" #--build=aarch64-unknown-linux-gnu
make -j 6
make -j 6 install
cd ..
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-smallcrush src/sr_testu01small.c
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-crush      src/sr_testu01.c
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-bigcrush   src/sr_testu01big.c



echo
echo "Building DieHarder"
echo
unzip -u external/dieharder-master.zip
cd dieharder-master
./autogen.sh
./configure
make
make install
cd ..



echo
echo "Installing to /usr/local"
echo
cp -f bin/* /usr/local/bin/



echo
echo "Cleaning up."
echo
rm -Rf PractRand-master bin dieharder-master TestU01-1.2.3

softrng install

echo "If you see !!! next to DieHarder it means it failed to compile."
echo "If you require its usage, you may have to install libtool first."
echo "Seen on macOS and fixed using 'brew install libtool'."
