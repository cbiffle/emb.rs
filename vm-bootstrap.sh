#!/bin/bash

set -ex

sudo apt-get update
sudo apt-get remove --purge landscape-common
sudo apt-get install -y software-properties-common
sudo add-apt-repository ppa:team-gcc-arm-embedded/ppa
sudo apt-get update
sudo apt-get install -y \
  gcc-arm-embedded=5-2016q3-1~trusty1 \
  build-essential \
  git \
  ninja-build

curl https://sh.rustup.rs -sSf > /tmp/rustup.sh
bash /tmp/rustup.sh -y
. ~/.cargo/env
cargo install xargo

cd /vagrant
rustup override add nightly
rustup component add rust-src

echo "cd /vagrant" >> ~/.profile
