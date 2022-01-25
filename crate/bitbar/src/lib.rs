#![deny(missing_docs, rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

#![cfg_attr(docsrs, feature(doc_cfg))]

//! This is `bitbar`, a library crate which includes helpers for writing BitBar plugins in Rust. BitBar is a system that makes it easy to add menus to the macOS menu bar. There are two apps implementing the BitBar system: [SwiftBar](https://swiftbar.app/) and [xbar](https://xbarapp.com/). This crate supports both of them, as well as [the discontinued original BitBar app](https://github.com/matryer/xbar/tree/a595e3bdbb961526803b60be6fd32dd0c667b6ec).
//!
//! There are two main entry points:
//!
//! * It's recommended to use the [`main`](crate::main) attribute and write a `main` function that returns a [`Menu`](crate::Menu), along with optional [`command`](crate::command) functions and an optional [`fallback_command`](crate::fallback_command) function.
//! * For additional control over your plugin's behavior, you can directly [`Display`](std::fmt::Display) a [`Menu`](crate::Menu).
//!
//! BitBar plugins must have filenames of the format `name.duration.extension`, even though macOS binaries normally don't have extensions. You will have to add an extension, e.g. `.o`, to make Rust binaries work as plugins.
//!
//! # Example
//!
//! ```rust
//! use bitbar::{Menu, MenuItem};
//!
//! #[bitbar::main]
//! fn main() -> Menu {
//!     Menu(vec![
//!         MenuItem::new("Title"),
//!         MenuItem::Sep,
//!         MenuItem::new("Menu Item"),
//!     ])
//! }
//! ```
//!
//! Or:
//!
//! ```rust
//! use bitbar::{Menu, MenuItem};
//!
//! fn main() {
//!     print!("{}", Menu(vec![
//!         MenuItem::new("Title"),
//!         MenuItem::Sep,
//!         MenuItem::new("Menu Item"),
//!     ]));
//! }
//! ```
//!
//! There is also [a list of real-world examples](https://github.com/fenhl/rust-bitbar#example-plugins).

use {
    std::{
        borrow::Cow,
        collections::BTreeMap,
        convert::TryInto,
        fmt,
        iter::FromIterator,
        process,
        vec,
    },
    url::Url,
};
pub use {
    bitbar_derive::{
        command,
        fallback_command,
        main,
    },
    crate::flavor::Flavor,
};
#[doc(hidden)] pub use { // used in proc macro
    inventory,
    notify_rust,
    structopt,
};
#[cfg(feature = "tokio")] #[doc(hidden)] pub use dep_tokio as tokio;
#[cfg(feature = "tokio02")] #[doc(hidden)] pub use dep_tokio02 as tokio;
#[cfg(feature = "tokio03")] #[doc(hidden)] pub use dep_tokio03 as tokio;

pub mod attr;
pub mod flavor;

/// A menu item that's not a separator.
#[derive(Debug, Default)]
pub struct ContentItem {
    /// This menu item's main content text.
    ///
    /// Any `|` in the text will be displayed as `¦`, and any newlines will be displayed as spaces.
    pub text: String,
    /// This menu item's alternate-mode menu item or submenu.
    pub extra: Option<attr::Extra>,
    /// Corresponds to BitBar's `href=` parameter.
    pub href: Option<Url>,
    /// Corresponds to BitBar's `color=` parameter.
    pub color: Option<attr::Color>,
    /// Corresponds to BitBar's `font=` parameter.
    pub font: Option<String>,
    /// Corresponds to BitBar's `size=` parameter.
    pub size: Option<usize>,
    /// Corresponds to BitBar's `bash=`, `terminal=`, `param1=`, etc. parameters.
    pub command: Option<attr::Command>,
    /// Corresponds to BitBar's `refresh=` parameter.
    pub refresh: bool,
    /// Corresponds to BitBar's `image=` or `templateImage=` parameter.
    pub image: Option<attr::Image>,
    /// Parameters for flavor-specific features.
    pub flavor_attrs: Option<flavor::Attrs>,
}

impl ContentItem {
    /// Returns a new menu item with the given text.
    ///
    /// Any `|` in the text will be displayed as `¦`, and any newlines will be displayed as spaces.
    pub fn new(text: impl ToString) -> ContentItem {
        ContentItem {
            text: text.to_string(),
            ..ContentItem::default()
        }
    }

    /// Adds a submenu to this menu item.
    pub fn sub(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.extra = Some(attr::Extra::Submenu(Menu::from_iter(items)));
        self
    }

    /// Adds a clickable link to this menu item.
    pub fn href(mut self, href: impl attr::IntoUrl) -> Result<Self, url::ParseError> {
        self.href = Some(href.into_url()?);
        Ok(self)
    }

    /// Sets this menu item's text color. Alpha channel is ignored.
    pub fn color<C: TryInto<attr::Color>>(mut self, color: C) -> Result<Self, C::Error> {
        self.color = Some(color.try_into()?);
        Ok(self)
    }

    /// Sets this menu item's text font.
    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = Some(font.to_string());
        self
    }

    /// Sets this menu item's font size.
    pub fn size(mut self, size: usize) -> Self {
        self.size = Some(size);
        self
    }

    /// Make this menu item run the given command when clicked.
    pub fn command<C: TryInto<attr::Command>>(mut self, cmd: C) -> Result<Self, C::Error> {
        self.command = Some(cmd.try_into()?);
        Ok(self)
    }

    /// Causes the BitBar plugin to be refreshed when this menu item is clicked.
    pub fn refresh(mut self) -> Self {
        self.refresh = true;
        self
    }

    /// Adds an alternate menu item, which is shown instead of this one as long as the option key ⌥ is held.
    pub fn alt(mut self, alt: impl Into<ContentItem>) -> Self {
        self.extra = Some(attr::Extra::Alternate(Box::new(alt.into())));
        self
    }

    /// Adds a template image to this menu item.
    pub fn template_image<T: TryInto<attr::Image>>(mut self, img: T) -> Result<Self, T::Error> {
        self.image = Some(attr::Image::template(img)?);
        Ok(self)
    }

    /// Adds an image to this menu item. The image will not be considered a template image unless specified as such by the `img` parameter.
    pub fn image<T: TryInto<attr::Image>>(mut self, img: T) -> Result<Self, T::Error> {
        self.image = Some(img.try_into()?);
        Ok(self)
    }

    fn render(&self, f: &mut fmt::Formatter<'_>, is_alt: bool) -> fmt::Result {
        // main text
        write!(f, "{}", self.text.replace('|', "¦").replace('\n', " "))?;
        // parameters
        let mut rendered_params = BTreeMap::default();
        if let Some(ref href) = self.href {
            rendered_params.insert(Cow::Borrowed("href"), Cow::Borrowed(href.as_ref()));
        }
        if let Some(ref color) = self.color {
            rendered_params.insert(Cow::Borrowed("color"), Cow::Owned(color.to_string()));
        }
        if let Some(ref font) = self.font {
            rendered_params.insert(Cow::Borrowed("font"), Cow::Borrowed(font));
        }
        if let Some(size) = self.size {
            rendered_params.insert(Cow::Borrowed("size"), Cow::Owned(size.to_string()));
        }
        if let Some(ref cmd) = self.command {
            //TODO (xbar) prefer “shell” over “bash”
            rendered_params.insert(Cow::Borrowed("bash"), Cow::Borrowed(&cmd.params.cmd));
            for (i, param) in cmd.params.params.iter().enumerate() {
                rendered_params.insert(Cow::Owned(format!("param{}", i + 1)), Cow::Borrowed(param));
            }
            if !cmd.terminal {
                rendered_params.insert(Cow::Borrowed("terminal"), Cow::Borrowed("false"));
            }
        }
        if self.refresh {
            rendered_params.insert(Cow::Borrowed("refresh"), Cow::Borrowed("true"));
        }
        if is_alt {
            rendered_params.insert(Cow::Borrowed("alternate"), Cow::Borrowed("true"));
        }
        if let Some(ref img) = self.image {
            rendered_params.insert(Cow::Borrowed(if img.is_template { "templateImage" } else { "image" }), Cow::Borrowed(&img.base64_data));
        }
        if let Some(ref flavor_attrs) = self.flavor_attrs {
            flavor_attrs.render(&mut rendered_params);
        }
        if !rendered_params.is_empty() {
            write!(f, " |")?;
            for (name, value) in rendered_params {
                let quoted_value = if value.contains(' ') {
                    Cow::Owned(format!("\"{}\"", value))
                } else {
                    value
                }; //TODO check for double quotes in value, fall back to single quotes? (test if BitBar supports these first)
                write!(f, " {}={}", name, quoted_value)?;
            }
        }
        writeln!(f)?;
        // additional items
        match &self.extra {
            Some(attr::Extra::Alternate(ref alt)) => { alt.render(f, true)?; }
            Some(attr::Extra::Submenu(ref sub)) => {
                let sub_fmt = format!("{}", sub);
                for line in sub_fmt.lines() {
                    writeln!(f, "--{}", line)?;
                }
            }
            None => {}
        }
        Ok(())
    }
}

impl fmt::Display for ContentItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.render(f, false)
    }
}

/// A menu item can either be a separator or a content item.
#[derive(Debug)]
pub enum MenuItem {
    /// A content item, i.e. any menu item that's not a separator.
    Content(ContentItem),
    /// A separator bar.
    Sep
}

impl MenuItem {
    /// Returns a new menu item with the given text. See `ContentItem::new` for details.
    pub fn new(text: impl fmt::Display) -> MenuItem {
        MenuItem::Content(ContentItem::new(text))
    }
}

impl Default for MenuItem {
    fn default() -> MenuItem {
        MenuItem::Content(ContentItem::default())
    }
}

impl From<ContentItem> for MenuItem {
    fn from(i: ContentItem) -> MenuItem {
        MenuItem::Content(i)
    }
}

impl fmt::Display for MenuItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MenuItem::Content(content) => write!(f, "{}", content),
            MenuItem::Sep => writeln!(f, "---")
        }
    }
}

/// A BitBar menu.
///
/// Usually constructed by calling [`collect`](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.collect) on an [`Iterator`](https://doc.rust-lang.org/std/iter/trait.Iterator.html) of `MenuItem`s.
#[derive(Debug, Default)]
pub struct Menu(pub Vec<MenuItem>);

impl Menu {
    /// Adds a menu item to the bottom of the menu.
    pub fn push(&mut self, item: impl Into<MenuItem>) {
        self.0.push(item.into());
    }
}

impl<A: Into<MenuItem>> FromIterator<A> for Menu {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Menu {
        Menu(iter.into_iter().map(Into::into).collect())
    }
}

impl<A: Into<MenuItem>> Extend<A> for Menu {
    fn extend<T: IntoIterator<Item = A>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(Into::into))
    }
}

impl IntoIterator for Menu {
    type Item = MenuItem;
    type IntoIter = vec::IntoIter<MenuItem>;

    fn into_iter(self) -> vec::IntoIter<MenuItem> { self.0.into_iter() }
}

/// This provides the main functionality of this crate: rendering a BitBar plugin.
///
/// Note that the output this generates already includes a trailing newline, so it should be used with `print!` instead of `println!`.
impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for menu_item in &self.0 {
            write!(f, "{}", menu_item)?;
        }
        Ok(())
    }
}

/// Members of this trait can be returned from a main function annotated with [`main`].
pub trait MainOutput {
    /// Converts this value into a [`Menu`], displaying the given template image in case of an error.
    fn main_output(self, error_template_image: Option<attr::Image>) -> Menu;
}

impl<T: Into<Menu>> MainOutput for T {
    fn main_output(self, _: Option<attr::Image>) -> Menu { self.into() }
}

/// In the `Err` case, the menu will be prefixed with a menu item displaying the `error_template_image` and the text `?`.
impl<T: MainOutput, E: MainOutput> MainOutput for Result<T, E> {
    fn main_output(self, error_template_image: Option<attr::Image>) -> Menu {
        match self {
            Ok(x) => x.main_output(error_template_image),
            Err(e) => {
                let mut header = ContentItem::new("?");
                if let Some(error_template_image) = error_template_image {
                    header = match header.template_image(error_template_image) {
                        Ok(header) => header,
                        Err(never) => match never {},
                    };
                }
                let mut menu = Menu(vec![header.into(), MenuItem::Sep]);
                menu.extend(e.main_output(None));
                menu
            }
        }
    }
}

/// Members of this trait can be returned from a subcommand function annotated with [`command`] or [`fallback_command`].
pub trait CommandOutput {
    /// Reports any errors in this command output as macOS notifications.
    fn report(self, cmd_name: &str);
}

impl CommandOutput for () {
    fn report(self, _: &str) {}
}

impl<T: CommandOutput, E: fmt::Display> CommandOutput for Result<T, E> {
    fn report(self, cmd_name: &str) {
        match self {
            Ok(x) => x.report(cmd_name),
            Err(e) => {
                notify(format!("{}: {}", cmd_name, e));
                process::exit(1);
            }
        }
    }
}

#[doc(hidden)] pub fn notify(body: impl fmt::Display) { // used in proc macro
    //let _ = notify_rust::set_application(&notify_rust::get_bundle_identifier_or_default("BitBar")); //TODO uncomment when https://github.com/h4llow3En/mac-notification-sys/issues/8 is fixed
    let _ = notify_rust::Notification::default()
        .summary(&env!("CARGO_PKG_NAME"))
        .sound_name("Funky")
        .body(&body.to_string())
        .show();
}
