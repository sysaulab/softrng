#!/bin/sh
#Run as root to install TestU01 to /usr/local/ and build the generic targets.

function cmd_path () {
  if [[ $ZSH_VERSION ]]; then
    whence -cp "$1" 2> /dev/null
  else  # bash
     type -P "$1"  # No output if not in $PATH
  fi
}

mkdir _testu01_
cd _testu01_

if [ $(cmd_path wget) ]; then
wget --output-document __extra__testu01.zip http://simul.iro.umontreal.ca/testu01/TestU01.zip;
elif [ $(cmd_path curl) ]; then
curl -L -o __extra__testu01.zip http://simul.iro.umontreal.ca/testu01/TestU01.zip
else
echo Please install curl or wget and try again.
fi

unzip __extra__testu01.zip
cd TestU01-1.2.3
#Uncomment the end of the next line to install on Linux on Arm64.
#Adjust if configure complain about unknown the build type
#We should look to make this more portable.
./configure --prefix="/usr/local" #--build=aarch64-unknown-linux-gnu
make -j 6
make -j 6 install

cd ../..

mkdir Release

cc -O2 -o Release/t-u01small src/sr_testu01small.c  -Iinclude -ltestu01 -lprobdist -lmylib -lm
cc -O2 -o Release/t-u01crush src/sr_testu01.c  -Iinclude -ltestu01 -lprobdist -lmylib -lm
cc -O2 -o Release/t-u01big src/sr_testu01big.c  -Iinclude -ltestu01 -lprobdist -lmylib -lm

mv Release/* /usr/local/bin/
rm -fR Release _testu01_
softrng refresh