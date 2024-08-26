#!/bin/sh
mkdir -p bin
mkdir -p logs

echo "Building SoftRNG"
echo "SOFTRNG INSTALL LOG (stdout)" > logs/softrng.log
echo "SOFTRNG ERROR LOG (stderr)" > logs/softrng.error
cc src/sr_timertest.c   -O2 -o ./bin/timertest >> logs/softrng.log 2>> logs/softrng.error
cc src/srs_password.c   -O2 -o ./bin/s-password >> logs/softrng.log 2>> logs/softrng.error
cc src/srf_bench.c      -O2 -o ./bin/f-peek >> logs/softrng.log 2>> logs/softrng.error
cc src/srs_fread.c      -O2 -o ./bin/s-file >> logs/softrng.log 2>> logs/softrng.error
cc src/srf_fwrite.c     -O2 -o ./bin/f-file >> logs/softrng.log 2>> logs/softrng.error
cc src/srf_hex.c        -O2 -o ./bin/f-hex >> logs/softrng.log 2>> logs/softrng.error
cc src/srf_limit.c      -O2 -o ./bin/f-limit >> logs/softrng.log 2>> logs/softrng.error
cc src/srt_hole.c       -O2 -o ./bin/t-hole >> logs/softrng.log 2>> logs/softrng.error
cc src/srf_xor.c        -O2 -o ./bin/f-xor >> logs/softrng.log 2>> logs/softrng.error
cc src/srg_qxo64.c      -O2 -o ./bin/f-qxo64 >> logs/softrng.log 2>> logs/softrng.error
cc src/srg_roxo64.c     -O2 -o ./bin/f-roxo64 >> logs/softrng.log 2>> logs/softrng.error
cc src/srs_zeros.c      -O2 -o ./bin/s-zeros >> logs/softrng.log 2>> logs/softrng.error
cc src/srs_getent.c     -O2 -o ./bin/s-getent >> logs/softrng.log 2>> logs/softrng.error
cc src/srs_icm64.c      -O2 -o ./bin/s-chaos >> logs/softrng.log 2>> logs/softrng.error
cc src/srt_bspec32.c    -O2 -o ./bin/t-bspec32 >> logs/softrng.log 2>> logs/softrng.error
cc src/softrng.c        -O2 -o ./bin/softrng >> logs/softrng.log 2>> logs/softrng.error



echo "Building PractRand"
echo "PRACTRAND INSTALL LOG (stdout)" > logs/practrand.log
echo "PRACTRAND ERROR LOG (stderr)" > logs/practrand.error
unzip -u external/PractRand-master.zip >> logs/practrand.log 2>> logs/practrand.error
cd PractRand-master/unix >> logs/practrand.log 2>> logs/practrand.error
make >> ../../logs/practrand.log 2>> ../../logs/practrand.error
cd ../.. >> ../../logs/practrand.log 2>> ../../logs/practrand.error
cp PractRand-master/bin/RNG_* ./bin >> logs/practrand.log 2>> logs/practrand.error



echo "Building TestU01"
echo "TESTU01 INSTALL LOG (stdout)" > logs/testu01.log
echo "TESTU01 ERROR LOG (stderr)" > logs/testu01.error
unzip -u external/TestU01.zip >> logs/testu01.log 2>> logs/testu01.error
cd TestU01-1.2.3 >> logs/testu01.log 2>> logs/testu01.error
#Uncomment the end of the next line to install on Linux/aarch64.
#Adjust if configure complain about "unknown build type"
#We should look to make this more portable.
./configure --prefix="/usr/local" >> ../logs/testu01.log 2>> ../logs/testu01.error #--build=aarch64-unknown-linux-gnu
make -j 6 >> ../logs/testu01.log 2>> ../logs/testu01.error
make -j 6 install >> ../logs/testu01.log 2>> ../logs/testu01.error
cd .. >> ../logs/testu01.log 2>> ../logs/testu01.error
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-smallcrush src/sr_testu01small.c >> logs/testu01.log 2>> logs/testu01.error
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-crush      src/sr_testu01.c >> logs/testu01.log 2>> logs/testu01.error
cc -O2 -Iinclude -ltestu01 -lprobdist -lmylib -lm -o bin/t-test-u01-bigcrush   src/sr_testu01big.c >> logs/testu01.log 2>> logs/testu01.error



echo "DieHarder"
echo "DIEHARDER INSTALL LOG (stdout)" > logs/dieharder.log
echo "DIEHARDER ERROR LOG (stderr)" > logs/dieharder.error
INPUT_STRING=$(uname -s)
  case $INPUT_STRING in
	"Darwin")
    unzip -u external/dieharder-master.zip >> logs/dieharder.log 2>> logs/dieharder.error
    cd dieharder-master >> logs/dieharder.log 2>> logs/dieharder.error
    ./autogen.sh >> ../logs/dieharder.log 2>> ../logs/dieharder.error
    ./configure >> ../logs/dieharder.log 2>> ../logs/dieharder.error
    make >> ../logs/dieharder.log 2>> ../logs/dieharder.error
    make install >> ../logs/dieharder.log 2>> ../logs/dieharder.error
    cd .. >> ../logs/dieharder.log 2>> ../logs/dieharder.error
		break
		;;
  "Linux")
    if [ -x "$(command -v apk)" ];       then sudo apk add --no-cache dieharder
    elif [ -x "$(command -v apt-get)" ]; then sudo apt-get install dieharder
    elif [ -x "$(command -v dnf)" ];     then sudo dnf install dieharder
    elif [ -x "$(command -v zypper)" ];  then sudo zypper install dieharder
    else echo "Please install dieharder manually.">&2; fi

	*)
		echo "Please install dieharder manually."
		;;
  esac



cp -f bin/* /usr/local/bin/
cp -Rf etc /etc/softrng
rm -Rf PractRand-master bin dieharder-master TestU01-1.2.3
chmod -Rf 777 logs
softrng install
echo "If you see Cleaning next to a module, it is missing."
echo "All installation logs are kept in the logs directory."
echo "You can report bugs and request assistance at:"
echo "https://github.com/sysaulab/softrng/issues"

