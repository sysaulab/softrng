#!/bin/bash

mkdir test_results_synthetic
cd test_results_synthetic



echo "TestU01 Small Crush : s-ent-synthetic|t-test-u01sc"
for i in {1..100}
do
echo "$i/100"
s-ent-synthetic|t-test-u01sc > small_crush.$i.txt
done



echo "TestU01 Small Crush : s-ent-synthetic|f-not|t-test-u01sc"
for i in {1..100}
do
echo "$i/100"
s-ent-synthetic|f-not|t-test-u01sc > small_crush_not.$i.txt
done



echo "TestU01 Small Crush : s-ent-synthetic|f-prng-qxo64|t-test-u01sc"
for i in {1..100}
do
echo "$i/100"
s-ent-synthetic|f-prng-qxo64|t-test-u01sc > small_crush_qxo.$i.txt
done



echo "TestU01 Big Crush : s-ent-synthetic|f-prng-qxo64|t-test-u01bc"
for i in {1..10}
do
echo "$i/10"
s-ent-synthetic|f-prng-qxo64|t-test-u01bc > big_crush.$i.txt
done



echo "Dieharder : s-ent-synthetic|f-prng-qxo64|t-test-diedarder"
for i in {1..10}
do
echo "$i/10"
s-ent-synthetic|f-prng-qxo64|t-test-diedarder > dieharder.$i.txt
done



echo "Practrand (stop at 32TB) : s-ent-synthetic|f-prng-qxo64|t-test-pr64mt"
for i in {1..1}
do
echo "$i/1"
s-ent-synthetic|f-prng-qxo64|t-test-pr64mt > practrand.$i.txt
done


