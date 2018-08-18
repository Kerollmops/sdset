#!/bin/sh

set -x

# This env variable is here to permit
# to use a custom `cargo bench` command if needed
CARGO_BENCH_CMD=${CARGO_BENCH_CMD:-cargo bench}

if [ $# -eq 1 ]; then
    $CARGO_BENCH_CMD > $1.bench
else
    echo 'Usage: gen_graphs.sh $BENCH_FILE'
    exit 1
fi

cargo run --manifest-path gen-bench-data/Cargo.toml -- $1.bench

gnuplot -e "benchname='difference'" graph.plt > misc/difference.png
gnuplot -e "benchname='intersection'" graph.plt > misc/intersection.png
gnuplot -e "benchname='union'" graph.plt > misc/union.png
gnuplot -e "benchname='symmetric_difference'" graph.plt > misc/symmetric_difference.png

echo "Graphs successfully generated!"
