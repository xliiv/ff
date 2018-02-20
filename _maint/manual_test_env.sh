#!/bin/sh

cargo build --release

sudo userdel -r ff-test
sudo adduser ff-test --gecos "First Last,RoomNumber,WorkPhone,HomePhone" --disabled-password

git clone https://github.com/xliiv/dot-files
sudo cp -r dot-files /home/ff-test/
sudo cp target/release/ff /home/ff-test/dot-files

sudo chown -R ff-test:ff-test /home/ff-test/*
ls -la /home/ff-test
