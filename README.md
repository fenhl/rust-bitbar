[![crates.io badge]][crates.io link]

This is `bitbar`, a library crate which includes helpers for writing [BitBar](https://getbitbar.com/) plugins in Rust. The main feature is the `Menu` type whose `Display` implementation generates output that conforms to the [BitBar plugin API](https://github.com/matryer/bitbar#plugin-api).

For a list of available Cargo features, see the crate-level documentation.

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
