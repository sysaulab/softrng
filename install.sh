#!/bin/sh
mkdir -p bin
mkdir -p logs

echo "Building SoftRNG"
echo "SOFTRNG INSTALL LOG (stdout)" > logs/softrng.std.txt
echo "SOFTRNG ERROR LOG (stderr)" > logs/softrng.err.txt

cc src/sr_timertest.c           -O2 -o ./bin/sr_timertest             >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/sr_rngoutputshim.c       -O2 -o ./bin/sr_rngoutputshim         >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/s-ent-getent.c           -O2 -o ./bin/s-ent-getent             >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/s-ent-password.c         -O2 -o ./bin/s-ent-password           >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/s-ent-synthetic.c        -O2 -o ./bin/s-ent-synthetic          >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/s-file.c                 -O2 -o ./bin/s-file                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/s-zeros.c                -O2 -o ./bin/s-zeros                  >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-peek.c                 -O2 -o ./bin/f-peek                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-not.c                  -O2 -o ./bin/f-not                    >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/t-file.c                 -O2 -o ./bin/f-file                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-copy.c                 -O2 -o ./bin/f-copy                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-hex.c                  -O2 -o ./bin/f-hex                    >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-limit.c                -O2 -o ./bin/f-limit                  >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/t-hole.c                 -O2 -o ./bin/t-hole                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-merge.c                -O2 -o ./bin/f-merge                  >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-fork.c                 -O2 -o ./bin/f-fork                   >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-prng-qxo64.c           -O2 -o ./bin/f-prng-qxo64             >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/f-prng-roxo64.c          -O2 -o ./bin/f-prng-roxo64            >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/t-test-bspec32.c         -O2 -o ./bin/t-test-bspec32           >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/t-test-bspec32-ref.c     -O2 -o ./bin/t-test-bspec32-ref       >> logs/softrng.std.txt   2>> logs/softrng.err.txt
cc src/softrng.c                -O2 -o ./bin/softrng                  >> logs/softrng.std.txt   2>> logs/softrng.err.txt



echo "Building PractRand"
echo "PRACTRAND INSTALL LOG (stdout)"                             > logs/practrand.std.txt
echo "PRACTRAND ERROR LOG (stderr)"                               > logs/practrand.err.txt
unzip -u external/PractRand-master.zip                            >> logs/practrand.std.txt         2>> logs/practrand.err.txt
cd PractRand-master/unix                                          >> logs/practrand.std.txt         2>> logs/practrand.err.txt
make                                                              >> ../../logs/practrand.std.txt   2>> ../../logs/practrand.err.txt
cd ../..                                                          >> ../../logs/practrand.std.txt   2>> ../../logs/practrand.err.txt
cp PractRand-master/bin/RNG_* ./bin                               >> logs/practrand.std.txt         2>> logs/practrand.err.txt



echo "Building TestU01"
echo "TESTU01 INSTALL LOG (stdout)"                               > logs/testu01.std.txt
echo "TESTU01 ERROR LOG (stderr)"                                 > logs/testu01.err.txt
unzip -u external/TestU01.zip                                     >> logs/testu01.std.txt         2>> logs/testu01.err.txt
cd TestU01-1.2.3                                                  >> logs/testu01.std.txt         2>> logs/testu01.err.txt
./configure --prefix="/usr/local"                                 >> ../logs/testu01.std.txt      2>> ../logs/testu01.err.txt
make -j 6                                                         >> ../logs/testu01.std.txt      2>> ../logs/testu01.err.txt
make -j 6 install                                                 >> ../logs/testu01.std.txt      2>> ../logs/testu01.err.txt
cd ..                                                             >> ../logs/testu01.std.txt      2>> ../logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-smallcrush src/t-test-u01-smallcrush.c    >> logs/testu01.std.txt 2>> logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-crush      src/t-test-u01-crush.c         >> logs/testu01.std.txt 2>> logs/testu01.err.txt
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-bigcrush   src/sr_testu01big.c            >> logs/testu01.std.txt 2>> logs/testu01.err.txt



cp -f bin/* /usr/local/bin/
cp -Rf softrng /etc
rm -Rf PractRand-master bin dieharder-master TestU01-1.2.3
chmod -Rf 777 logs
softrng install
echo "You can look into the logs folder to help resolve eventual problems."
echo "Please provide a copy of your logs in a zip file when reporting a bug."
echo 
echo "To file a bug report : https://github.com/sysaulab/softrng/issues"
echo "Any other assistance : https://github.com/sysaulab/softrng/discussions"
echo

