# 0.5.2

Documentation fixes.

# 0.5.1

Documentation fixes.

# 0.5.0

* `main` functions may optionally take a `Flavor` argument
* Types used for configuring menu items have been moved to a new `attr` module
* `Params` is now an opaque type
* New `ContentItem::size` method to set font size
* SwiftBar only: Support for commands with more than 5 parameters
* SwiftBar only: Support for using different colors depending on whether the system is in dark mode
* SwiftBar only: Support for adding an SF Symbols image to a menu item

# 0.4.4

* Added the `flavor` module
* Added the `assume-flavor` feature

# 0.4.3

* The `tokio` feature now uses `tokio` 1; added the `tokio03` feature to use `tokio` 0.3 instead
* Upgraded the optional `serenity` dependency to 0.10
* `command` functions may be async (requires one of the tokio features)
* Added the `fallback_command` attribute macro
* Added the `MainOutput` trait; `main` functions may return any member of it
* Added the `CommandOutput` trait; `command` and `fallback_command` functions may return any member of it
* This crate is now `#![forbid(unsafe_code)]`

# 0.4.2

Documentation fixes.

# 0.4.1

* `main` functions may return plain `Menu` rather than only `Result<Menu, _>`
* Added an optional `error_template_image` parameter to `main` attribute
* The `tokio` 0.3 dependency is now optional (enabled by default), and a `tokio02` feature has been added to use `tokio` 0.2 instead

# 0.4.0

* Added the `main` and `command` attribute macros
* Added `Extend<A>` implementations for `Menu` (where `A: Into<MenuItem>`)
* Added an `IntoIterator` implementation for `Menu`

# 0.3.1

* Added an `impl From<Vec<u8>>` implementation for `Image` (requires `base64` feature)
