#!/bin/sh
#Run as root to install TestU01 and build the stdin shims..

mkdir _testu01_
cd _testu01_
mkdir install_tree
cd install_tree
basedir=`.`
cd ..

if ! command -v curl &> /dev/null
then
    if ! command -v wget &> /dev/null
    then
        echo "Please install curl or wget and try again."
        exit 1
    else
    wget --output-document __extra__testu01.zip http://simul.iro.umontreal.ca/testu01/TestU01.zip
    fi
else
curl -L -o __extra__testu01.zip http://simul.iro.umontreal.ca/testu01/TestU01.zip
fi

unzip __extra__testu01.zip
cd TestU01-1.2.3
./configure --prefix="$basedir"
make -j 6
#make -j 6 install
#cd ..
#    mkdir lib-so
#    mv lib/*.so lib-so/.