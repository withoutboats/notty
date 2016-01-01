#!/bin/sh
# This script tests both the natty and scaffolding crates, to make sure that
# changes to natty's public API do not break the scaffolding terminal

test_subcrate() {
    cd $1 && cargo test && cd ..
}

cargo test && test_subcrate 'notty-cairo' && test_subcrate 'scaffolding'
