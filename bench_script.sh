#!/bin/sh

# This env variable is here to permit
# to use a custom `cargo bench` command if needed
CARGO_BENCH_CMD=${CARGO_BENCH_CMD:-cargo bench --features unstable}

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
        NEW="latest"
    fi
}

exit_if_dirty
if [[ "$NEW" = latest ]]; then
    $CARGO_BENCH_CMD > $NEW.bench
    else
    if [ ! -f $NEW.bench ]; then
        git checkout $NEW
        $CARGO_BENCH_CMD > $NEW.bench
        git checkout -
    fi
fi

if [ ! -f $OLD.bench ]; then
    git checkout $OLD
    $CARGO_BENCH_CMD > $OLD.bench
    git checkout -
fi

cargo benchcmp --threshold 5 $OLD.bench $NEW.bench
