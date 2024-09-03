#!/bin/sh
mkdir -p logs

echo "Building TestU01"
echo "TESTU01 INSTALL LOG (stdout)"
echo "TESTU01 ERROR LOG (stderr)"
rm -Rf TestU01-1.2.3
unzip TestU01.zip
cd TestU01-1.2.3
./configure --prefix="/usr/local"
make -j 6
make -j 6 install
cd ..
rm -Rf TestU01-1.2.3
