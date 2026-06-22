#!/bin/sh
echo "Please note the following value after it stabilizes. (f-prng-qxo64)"
f-prng-qxo64 < /dev/random | f-peek > /dev/null 
