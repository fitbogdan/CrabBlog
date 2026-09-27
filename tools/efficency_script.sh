#!/bin/bash
# Usage: ./efficency_script.sh [runs] [route]
# Defaults: 10 runs on /style.css
# Example: ./efficency_script.sh 5 /post/2

export LC_NUMERIC=C

runs=${1:-10}
route=${2:-/page/2}
url="http://127.0.0.1:8080$route"

echo "Stress testing server: $runs times on $route"

results=()

for i in $(seq 1 "$runs")
do
    echo "Tests: $i/$runs"
    result=$(ab -q -n 10000 -c 1000 "$url" | grep seconds | awk '{print $5}');
    results+=("$result")
done

avg=$(printf '%s\n' "${results[@]}" | awk '{sum+=$1} END{printf "%.4f\n", sum/NR}')

echo "Average result: $avg";
echo "Printing into benchmarks..."

timestampdate=$(date '+%Y-%m-%d')
timestamptime=$(date '+%H:%M:%S')

echo "Time: $timestamptime, Runs: $runs, Speed (Seconds): $avg, Date: $timestampdate, Route: $route" >> "benchmarks/speed.txt"
