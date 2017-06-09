#!/bin/bash
set -e

SCRIPT_DIR="`dirname \"$0\"`"
source $SCRIPT_DIR/_utils.sh

FF_LATEST_FF_URL="${FF_LATEST_FF_URL:-"https://github.com/xliiv/ff/releases/latest"}"

cmt "Let's say you want to have \`dot-files\` dir as a git repo."
exe cd ~
exe mkdir dot-files
exe cd dot-files
exe git init
cmt "Now we need to download \`ff\` binary"
exe wget $FF_LATEST_FF_URL
exe chmod +x ff
cmt "Now we want to add \`.bashrc\` to \`dot-files\`"
exe cd
exe ~/dot-files/ff add .bashrc
cmt "Oops, we haven't told \`ff\` yet where is the \`dot-files\` dir"
exe ~/dot-files/ff init dot-files
cmt "Ok, now it should work.."
exe ~/dot-files/ff add .bashrc
cmt "Let's take a look at our home dir, \`.bashrc\` should be symlinked"
exe ls -la
cmt "If you are satisfied with changes in \`dot-files\` repo. - commit and push"
cmt "You can also revert \`ff add\` operation by .."
exe ~/dot-files/ff remove ~/.bashrc
cmt ".. and again if the change is ok - commit and push"