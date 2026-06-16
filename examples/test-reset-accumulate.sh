#!/bin/sh

while :
do
	s-seedy64-reset | f-limit 10MiB | f-peek >> reset.bin
done
