#!/bin/sh

set -x

# This env variable is here to permit
# to use a custom `cargo bench` command if needed
CARGO_BENCH_CMD=${CARGO_BENCH_CMD:-cargo bench}

COMMIT=$(git rev-parse --short 'HEAD')

if [ ! -f $COMMIT.bench ]; then
    $CARGO_BENCH_CMD > $COMMIT.bench
fi

cargo run --manifest-path gen-bench-data/Cargo.toml -- $COMMIT.bench

gnuplot -e "benchname='difference'" graph.plt > misc/difference.png
gnuplot -e "benchname='intersection'" graph.plt > misc/intersection.png
gnuplot -e "benchname='union'" graph.plt > misc/union.png

echo "Graphs successfully generated!"
