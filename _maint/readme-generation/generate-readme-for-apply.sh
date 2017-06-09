#!/bin/bash
set -e

SCRIPT_DIR="`dirname \"$0\"`"
source $SCRIPT_DIR/_utils.sh

cmt "Let's say you have dot-files repo. on github.."
exe git clone https://github.com/xliiv/dot-files
exe cd dot-files
exe ls -la ~
exe ./ff init
exe ./ff apply
cmt "That's it.. . Now each file in your \`dot-files\` repo. is a symlink to its counterpart in your home dir"
exe ls -la ~
