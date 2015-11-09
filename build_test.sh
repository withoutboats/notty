#!/bin/sh
# This script tests both the natty and anterminal crates, to make sure that
# changes to natty's public API do not break anterminal.

cargo test && cd anterminal && cargo test && cd ..
