#!/usr/bin/env bash

export DISPLAY=:0
export PATH="$HOME/.cargo/bin:$PATH"
export RUST_TARGET_PATH=$(pwd)/kernel
make run
