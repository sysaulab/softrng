#!/bin/sh
s-seedy64 | f-file seeqxoseed.bin | f-prng-qxo64 | f-peek | RNG_test stdin64 -multithreaded
