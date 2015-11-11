#!/bin/sh
# This script tests both the natty and scaffolding crates, to make sure that
# changes to natty's public API do not break the scaffolding terminal

cargo test && cd scaffolding && cargo test && cd ..
