#!/bin/sh
set -e

cp ../target/debug/ff ./

docker build \
    --build-arg FF_TEST_PATH=$FF_TEST_PATH \
    --build-arg FF_TEST_SHOW_STDOUT=$FF_TEST_SHOW_STDOUT \
    -t ff-test . \
    --no-cache --rm --force-rm
