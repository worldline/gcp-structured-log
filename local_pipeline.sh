#!/bin/sh

cargo watch \
  -c \
  -s 'cargo check --message-format short' \
  -s 'cargo clippy --color always --all-targets' \
  -x build \
  -x test \
  -s 'cargo build -p gcp-log --release'
