#!/bin/sh
#Run as root to download, compile and install the https://github.com/MartyMacGyver distribution of PractRand
#Tested under macos and Linux.


function cmd_path () {
  if [[ $ZSH_VERSION ]]; then
    whence -cp "$1" 2> /dev/null
  else  # bash
     type -P "$1"  # No output if not in $PATH
  fi
}

if [ $(cmd_path wget) ]; then
wget --output-document __extra__practrand.zip https://github.com/MartyMacGyver/PractRand/zipball/master;
elif [ $(cmd_path curl) ]; then
curl -L -o __extra__practrand.zip https://github.com/MartyMacGyver/PractRand/zipball/master
else
echo Please install curl or wget and try again.
fi

unzip __extra__practrand.zip
cd MartyMacGyver-PractRand-6ae26d8/unix
make
cd ../..
cp MartyMacGyver-PractRand-6ae26d8/bin/RNG_* /usr/local/bin
rm -Rf MartyMacGyver-PractRand-6ae26d8 __extra__practrand.zip

softrng refresh