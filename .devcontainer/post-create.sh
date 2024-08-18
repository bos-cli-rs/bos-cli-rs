#!/bin/bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

sudo apt update
sudo apt install -y libudev-dev

git submodule init
git submodule update
