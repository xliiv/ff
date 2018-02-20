#!/bin/sh

set -e

if [ $# -eq 0 ]
then
    echo "Version required, eg.: v1.0.0"
    exit 1
fi

TAG_NAME=$1
DST_NAME="ff-$TAG_NAME-x86_64-unknown-linux-gnu.tar.gz"

cd ..
cargo build --release
cd _maint
tar -czvf $DST_NAME -C ../target/release ff
# TODO: use it in travis
