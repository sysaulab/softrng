#!/bin/sh
#Run as root to install Dieharder in Linux Debian. May work in other apt based distributions.

mkdir _testu01_
cd _testu01_
basedir=`pwd`
curl -OL http://simul.iro.umontreal.ca/testu01/TestU01.zip
unzip TestU01.zip
cd TestU01-1.2.3
./configure --prefix="$basedir"
make -j 6
make -j 6 install
cd ..
    mkdir lib-so
    mv lib/*.so lib-so/.