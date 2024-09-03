#!/bin/sh
echo "Building SoftRNG"

if [ -e /usr/local/bin/RNG_test ]
then
 echo "PractRand is already installed."
else
 echo "Building PractRand"
 cd external
 sudo ./install_practrand.sh
 cd ..
fi

if [ -e /usr/local/include/TestU01.h ]
then
 echo "TestU01 is already installed."
else
 echo "Building PractRand"
 cd external
 sudo ./install_testu01.sh
 cd ..
fi

c99 src/sr_timertest.c -O2 -o /usr/local/bin/sr_timertest 
c99 src/sr_rngoutputshim.c -O2 -o /usr/local/bin/sr_rngoutputshim 
c99 src/s-ent-getent.c -O2 -o /usr/local/bin/s-ent-getent 
c99 src/s-ent-password.c -O2 -o /usr/local/bin/s-ent-password 
c99 src/s-ent-synthetic.c -lpthread -O2 -o /usr/local/bin/s-ent-synthetic
c99 src/s-file.c -O2 -o /usr/local/bin/s-file 
c99 src/s-zeros.c -O2 -o /usr/local/bin/s-zeros 
c99 src/f-peek.c -O2 -o /usr/local/bin/f-peek 
c99 src/f-not.c -O2 -o /usr/local/bin/f-not 
c99 src/f-file.c -O2 -o /usr/local/bin/f-file 
c99 src/f-copy.c -O2 -o /usr/local/bin/f-copy 
c99 src/f-hex.c -O2 -o /usr/local/bin/f-hex 
c99 src/f-limit.c -O2 -o /usr/local/bin/f-limit 
c99 src/t-hole.c -O2 -o /usr/local/bin/t-hole 
c99 src/f-merge.c -O2 -o /usr/local/bin/f-merge 
c99 src/t-test-all64.c -O2 -o /usr/local/bin/t-test-all64
c99 src/f-fork.c -O2 -o /usr/local/bin/f-fork
c99 src/f-prng-qxo64.c -O2 -o /usr/local/bin/f-prng-qxo64
c99 src/f-prng-roxo64.c -O2 -o /usr/local/bin/f-prng-roxo64
c99 src/t-test-bspec32.c -O2 -o /usr/local/bin/t-test-bspec32
c99 src/t-test-bspec32-ref.c -O2 -o /usr/local/bin/t-test-bspec32-ref
c99 src/softrng.c -O2 -o /usr/local/bin/softrng 
c99 src/autoseed64.c -O2 -o /usr/local/bin/autoseed
c99 src/prandom.c -O2 -o /usr/local/bin/prandom
c99 -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o /usr/local/bin/t-test-u01sc src/t-test-u01-smallcrush.c
c99 -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o /usr/local/bin/t-test-u01c src/t-test-u01-crush.c 
c99 -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o /usr/local/bin/t-test-u01bc src/t-test-u01-bigcrush.c 

cp -Rf softrng /etc
softrng install

echo 
echo "If you see errors, you may have forgot to run the install script as root."
echo "https://github.com/sysaulab/softrng/"
echo
