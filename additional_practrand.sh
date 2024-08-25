#!/bin/sh
#Run as root to download, compile and install the https://github.com/MartyMacGyver distribution of PractRand
#Tested under macos and Linux.

curl -L -o __extra__practrand.zip https://github.com/MartyMacGyver/PractRand/zipball/master
unzip __extra__practrand.zip
cd MartyMacGyver-PractRand-6ae26d8/unix
make
cd ../..
cp MartyMacGyver-PractRand-6ae26d8/bin/RNG_* /usr/local/bin
rm -Rf MartyMacGyver-PractRand-6ae26d8 __extra__practrand.zip