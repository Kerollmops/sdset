#!/bin/sh

# This env variable is here to permit
# to use a custom `cargo bench` command if needed
CARGO_BENCH_CMD=${CARGO_BENCH_CMD:-cargo bench}

if [ $# -eq 0 ]; then
    echo "comparing benchmarks of HEAD~1 and HEAD..."
    OLD=$(git rev-parse --short 'HEAD~1')
    NEW=$(git rev-parse --short 'HEAD')
elif [ $# -eq 1 ]; then
    echo "comparing benchmarks of $1 and HEAD..."
    OLD=$(git rev-parse --short $1)
    NEW=$(git rev-parse --short 'HEAD')
elif [ $# -eq 2 ]; then
    echo "comparing benchmarks of $1 and $2..."
    OLD=$(git rev-parse --short $1)
    NEW=$(git rev-parse --short $2)
else
    echo 'Usage: bench_script.sh [$OLD] [$NEW]'
    exit 1
fi

exit_if_dirty() {
    if ! git diff-files --quiet; then
        echo 'Your repository must not be dirty'
        exit 1
    fi
}

if [ ! -f $NEW.bench ]; then
    exit_if_dirty
    git checkout $NEW
    $CARGO_BENCH_CMD > $NEW.bench
    git checkout -
fi

if [ ! -f $OLD.bench ]; then
    exit_if_dirty
    git checkout $OLD
    $CARGO_BENCH_CMD > $OLD.bench
    git checkout -
fi

cargo benchcmp --threshold 5 $OLD.bench $NEW.bench
