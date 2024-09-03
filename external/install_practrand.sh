#!/bin/sh
mkdir -p logs

echo "Building PractRand"
echo "PRACTRAND INSTALL LOG (stdout)"
echo "PRACTRAND ERROR LOG (stderr)"
rm -Rf PractRand-master
unzip -u PractRand-master.zip
cd PractRand-master/unix
make
cd ../..
cp -f PractRand-master/bin/RNG_* /usr/local/bin
rm -Rf PractRand-master