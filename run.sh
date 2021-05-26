#!/bin/bash

cargo clippy \
  && cargo build \
  && (rm tests/images/output/*.png || echo "no test-generated images to remove") \
  && CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind" \
     cargo test --release $@ \
  && cargo doc \
  && cp -r target/doc /mnt/c/Users/cmaza/Documents/
