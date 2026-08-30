#!/bin/bash


export LC_NUMERIC=C


echo "Stress testing server: $1 times"

sum=0
results=()

for i in $(seq 1 "$1")
do
    echo "Tests: $i/$1"
    result=$(ab -q -n 10000 -c 1000 http://127.0.0.1:8080/ | grep seconds | awk '{print $5}');
    results+=("$result")
done

avg=$(printf '%s\n' "${results[@]}" | awk '{sum+=$1} END{printf "%.4f\n", sum/NR}')

echo "Average result: $avg";
echo "Printing into benchmarks..."

timestampdate=$(date '+%Y-%m-%d')
timestamptime=$(date '+%H:%M:%S')

echo "Time: $timestamptime, Runs: $1, Speed (Seconds): $avg, Date: $timestampdate" >> "benchmarks/speed.txt"