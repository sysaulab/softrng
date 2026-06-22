#!/bin/sh
echo "Filling with slow data first (reset mode)...\n"
s-seedy64-reset | f-peek | f-file seeqxoseed_reset.bin | f-prng-qxo64 | f-peek | RNG_test stdin64 -multithreaded
