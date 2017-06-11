#!/bin/bash
set -e

if [ -z ${FF_TAG_NAME+x} ]
then
    echo "Set os env FF_TAG_NAME, like: v1.0.0"
    exit 1
fi

SCRIPT_DIR="`dirname \"$0\"`"
source $SCRIPT_DIR/_utils.sh

FF_GZIP_FILENAME="ff-$FF_TAG_NAME-x86_64-unknown-linux-gnu.tar.gz"
FF_LATEST_FF_URL="https://github.com/xliiv/ff/releases/download/$FF_TAG_NAME/$FF_GZIP_FILENAME"


cmt "Let's say you want to have \`dot-files\` dir as a git repo."
exe cd ~
exe mkdir dot-files
exe cd dot-files
exe git init
cmt "Now we need to download \`ff\` binary"
exe wget -q $FF_LATEST_FF_URL
exe tar -xvzf $FF_GZIP_FILENAME
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