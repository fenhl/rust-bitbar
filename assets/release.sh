#!/usr/local/bin/zsh

set -e

cd crate/bitbar-derive
cargo publish
cd ../cargo-bitbar
cargo publish
cd ../bitbar
cargo publish
