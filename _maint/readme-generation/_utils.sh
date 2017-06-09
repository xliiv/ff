#!/bin/bash
set -e

#function to display comments
cmt() { echo "\$ #" "$@" ; }

#function to display & execute commands
exe() { echo "\$ $@" ; "$@" ; }
