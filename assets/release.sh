#!/usr/local/bin/zsh

set -e

cd crate/bitbar-derive
cargo publish
#TODO only release cargo-bitbar if changed
cd ../cargo-bitbar
cargo publish
cd ../bitbar
cargo publish
