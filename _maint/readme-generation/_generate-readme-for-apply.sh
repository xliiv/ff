#!/bin/bash
set -e

SCRIPT_DIR="`dirname \"$0\"`"
source $SCRIPT_DIR/_utils.sh

cmt "Let's say you already have dot-files repo. on github.."
cmt ".. and you want to use in .."
exe git clone https://github.com/xliiv/dot-files
exe cd dot-files
cmt "We need to tell \`ff\` that this is our \`dot-files\` dir"
exe ./ff init --dir-path .
cmt "Now we are ready to replace home dir files with files contained by \`dot-files\` dir"
exe ./ff apply --sync-subdir .
cmt "That's it.. . Now each file in your \`dot-files\` repo. is a symlink to its counterpart in your home dir"
cmt "Take a look.. "
exe ls -la ~
cmt "see? :)"
