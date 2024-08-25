#!/bin/sh
#Run as root to download, compile and install the https://github.com/MartyMacGyver distribution of PractRand
#Tested under macos and Linux.

if ! command -v curl &> /dev/null
then
    if ! command -v wget &> /dev/null
    then
        echo "Please install curl or wget and try again."
        exit 1
    else
    wget --output-document __extra__practrand.zip https://github.com/MartyMacGyver/PractRand/zipball/master
    fi
else
curl -L -o __extra__practrand.zip https://github.com/MartyMacGyver/PractRand/zipball/master
fi
unzip __extra__practrand.zip
cd MartyMacGyver-PractRand-6ae26d8/unix
make
cd ../..
cp MartyMacGyver-PractRand-6ae26d8/bin/RNG_* /usr/local/bin
rm -Rf MartyMacGyver-PractRand-6ae26d8 __extra__practrand.zip