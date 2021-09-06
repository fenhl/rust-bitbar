[![crates.io badge]][crates.io link] [![docs.rs badge]][docs.rs link]

This is `bitbar`, a library crate which includes helpers for writing BitBar plugins in Rust. BitBar is a system that makes it easy to add menus to the macOS menu bar. There are two apps implementing the BitBar system: [SwiftBar](https://swiftbar.app/) and [xbar](https://xbarapp.com/). This crate supports both of them, as well as [the discontinued original BitBar app](https://github.com/matryer/xbar/tree/a595e3bdbb961526803b60be6fd32dd0c667b6ec).

# Example plugins

Here are some BitBar plugins that use this library:

* [BitBar version](https://github.com/fenhl/bitbar-version)
* [Mediawiki watchlist](https://github.com/fenhl/bitbar-mediawiki-watchlist)
* [speedrun.com](https://github.com/fenhl/bitbar-speedruncom)
* [twitch.tv](https://github.com/fenhl/bitbar-twitch)
* [Wurstmineberg server status](https://github.com/wurstmineberg/bitbar-server-status)

If you have a BitBar plugin that uses this library, feel free to open a pull request to add it to this list.

[crates.io badge]: https://img.shields.io/crates/v/bitbar.svg?style=flat-square
[crates.io link]: https://crates.io/crates/bitbar
[docs.rs badge]: https://img.shields.io/badge/docs-online-dddddd.svg?style=flat-square
[docs.rs link]: https://docs.rs/bitbar
