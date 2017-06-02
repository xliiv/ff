#!/bin/sh
set -e

cp ../target/debug/ff ./
docker build -t ff-test . --no-cache --rm --force-rm