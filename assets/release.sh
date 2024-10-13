#!/usr/bin/env zsh

set -e

git push
cd crate/bitbar-derive
cargo publish
#TODO only release cargo-bitbar if changed
#cd ../cargo-bitbar
#cargo publish
cd ../bitbar
cargo publish
