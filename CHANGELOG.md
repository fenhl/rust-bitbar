# 0.9.0

* Improved error notifications for subcommands
* **Breaking:** The `CommandOutput` implementation requires the error type to implement `Debug`

# 0.8.2

* Fix `SwiftBar::checked` not doing anything

# 0.8.1

* New `SwiftBar::checked` method

# 0.8.0

* **Breaking:** A command declared using `#[bitbar::command]` must also be registered via `#[bitbar::main(commands(...))]`
* **Breaking:** A fallback command declared using `#[bitbar::fallback_command]` must also be registered via `#[bitbar::main(fallback_command = "...")]`
* **Breaking:** The `tokio02` and `tokio03` features have been removed
* **Breaking:** Upgraded the optional `image` dependency from 0.22 to 0.24
* **Breaking:** Upgraded the optional `serenity` dependency from 0.10 to 0.11

# 0.7.3

* New `swiftbar::Notification::command` method

# 0.7.2

* New `SwiftBar::plugin_name` method
* New `swiftbar::Notification` type

# 0.7.1 (`cargo-bitbar` 0.1.1)

* Support for trailing stream separators in the newest SwiftBar beta (see [swiftbar/SwiftBar#273](https://github.com/swiftbar/SwiftBar/issues/273) for details)

# 0.7.0 (`cargo-bitbar` 0.1.0)

* **Breaking:** The `MainOutput` trait now prints the menu instead of returning it
* `cargo bitbar` is a new `cargo` subcommand that can add plugin metadata to binary SwiftBar plugins
* Support for streamable SwiftBar plugins via `bitbar::flavor::swiftbar::BlockingStream` and (with one of the tokio features) `bitbar::flavor::swiftbar::Stream`
* New `AsyncMainOutput` trait if printing the menu requires `async` (requires one of the tokio features)

# 0.6.0

* **Breaking:** `command` functions now take any number of parameters that will be parsed from command-line args; use `#[command(varargs)]` to take a `Vec<String>` instead
* **Breaking:** The `fallback_command` function now takes the command name as a `String` and the remaining arguments as a `Vec<String>`
* `command` functions now generate functions that return `Params`
* Added a `push` method to `Menu`

# 0.5.2

Documentation fixes.

# 0.5.1

Documentation fixes.

# 0.5.0

* **Breaking:** Types used for configuring menu items have been moved to a new `attr` module
* **Breaking:** `Params` is now an opaque type
* `main` functions may optionally take a `Flavor` argument
* New `ContentItem::size` method to set font size
* SwiftBar only: Support for commands with more than 5 parameters
* SwiftBar only: Support for using different colors depending on whether the system is in dark mode
* SwiftBar only: Support for adding an SF Symbols image to a menu item

# 0.4.4

* Added the `flavor` module
* Added the `assume-flavor` feature

# 0.4.3

* **Breaking:** The `tokio` feature now uses `tokio` 1; added the `tokio03` feature to use `tokio` 0.3 instead
* **Breaking:** Upgraded the optional `serenity` dependency from 0.9 to 0.10
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

* **Breaking:** Upgraded the optional `serenity` dependency from 0.7 to 0.9
* Added the `main` and `command` attribute macros
* Added `Extend<A>` implementations for `Menu` (where `A: Into<MenuItem>`)
* Added an `IntoIterator` implementation for `Menu`

# 0.3.1

* Added an `impl From<Vec<u8>>` implementation for `Image` (requires `base64` feature)
