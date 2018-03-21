#!/bin/sh
set -e

if [ -z ${FF_TAG_NAME+x} ]
then
    echo "Set os env FF_TAG_NAME, like: v1.0.0"
    exit 1
fi

docker build \
    --build-arg FF_TAG_NAME=$FF_TAG_NAME \
    -t ff-readme . \
    --no-cache --rm --force-rm
