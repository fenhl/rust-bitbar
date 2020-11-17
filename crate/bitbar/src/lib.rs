#![deny(rust_2018_idioms, unused, unused_import_braces, unused_qualifications, warnings, missing_docs)]

//! This is `bitbar`, a library crate which includes helpers for writing [BitBar](https://getbitbar.com/) plugins in Rust. The main feature is the `Menu` type whose `Display` implementation generates output that conforms to the [BitBar plugin API](https://github.com/matryer/bitbar#plugin-api).
//!
//! # Features
//!
//! The following features can be enabled via Cargo:
//!
//! * `base64`: Adds a depencency to the [`base64`](https://crates.io/crates/base64) crate and implements conversion methods from PNG files that aren't already base64-encoded to `Image`s.
//! * `css-colors`: Adds a dependency to the [`css-colors`](https://crates.io/crates/css-colors) crate and implements `IntoColor` for its color types `RGB`, `RGBA`, `HSL`, and `HSLA`.
//! * `image`: Adds a depencency to the [`image`](https://crates.io/crates/image) crate. If the `base64` feature is also enabled, implements `TryFrom<DynamicImage>` for `Image`.
//! * `serenity`: Adds a dependency to the [`serenity`](https://crates.io/crates/serenity) crate and implements `IntoColor` for its `Colour` type.
//! * `url1`: Adds a dependency to the outdated version 1 of the [`url`](https://crates.io/crates/url) crate and implements `IntoUrl` for its `Url` type.
//!
//! # Example
//!
//! ```rust
//! use bitbar::{Menu, MenuItem};
//!
//! fn main() {
//!     print!("{}", Menu(vec![
//!         MenuItem::new("Title"),
//!         MenuItem::Sep,
//!         MenuItem::new("Menu Item")
//!     ]));
//! }
//! ```

use {
    std::{
        collections::BTreeMap,
        convert::{
            TryFrom,
            TryInto,
        },
        fmt,
        iter::FromIterator,
        vec,
    },
    css_color_parser::{
        Color,
        ColorParseError,
    },
    url::Url,
};
#[cfg(all(feature = "base64", feature = "image"))]
use {
    image::{
        DynamicImage,
        ImageError,
        ImageOutputFormat::PNG,
        ImageResult,
    },
};
#[cfg(feature = "url1")]
use url1::Url as Url1;
pub use bitbar_derive::{
    command,
    main,
};
#[doc(hidden)] pub use { // used in proc macro
    inventory,
    notify_rust,
    structopt,
    tokio,
};

#[derive(Debug)]
/// A menu item's alternate mode or submenu.
pub enum Extra {
    /// A menu item's alternate mode, shown when <key>⌥</key> is held.
    Alternate(Box<ContentItem>), //TODO make sure alts don't have submenus
    /// A submenu.
    Submenu(Menu)
}

/// Used by `ContentItem::color`.
pub trait IntoColor {
    /// Converts `self` into a [`Color`](https://docs.rs/css-color-parser/0.1.2/css_color_parser/struct.Color.html).
    fn into_color(self) -> Result<Color, ColorParseError>;
}

impl IntoColor for &str {
    fn into_color(self) -> Result<Color, ColorParseError> {
        Ok(self.parse()?)
    }
}

impl IntoColor for Color {
    fn into_color(self) -> Result<Color, ColorParseError> {
        Ok(self)
    }
}

#[cfg(feature = "css-colors")]
macro_rules! impl_into_color_for_css_color {
    ($t:ty) => {
        impl IntoColor for $t {
            fn into_color(self) -> Result<Color, ColorParseError> {
                Ok(self.to_string().parse()?)
            }
        }
    };
}

#[cfg(feature = "css-colors")] impl_into_color_for_css_color!(css_colors::RGB);
#[cfg(feature = "css-colors")] impl_into_color_for_css_color!(css_colors::RGBA);
#[cfg(feature = "css-colors")] impl_into_color_for_css_color!(css_colors::HSL);
#[cfg(feature = "css-colors")] impl_into_color_for_css_color!(css_colors::HSLA);

#[cfg(feature = "serenity")]
impl IntoColor for serenity::utils::Colour {
    fn into_color(self) -> Result<Color, ColorParseError> {
        Ok(Color {
            r: self.r(),
            g: self.g(),
            b: self.b(),
            a: 1.0
        })
    }
}

/// Used by `ContentItem::href`.
pub trait IntoUrl {
    /// Converts `self` into a [`Url`](https://docs.rs/url/2/url/struct.Url.html).
    fn into_url(self) -> Result<Url, url::ParseError>;
}

impl IntoUrl for Url {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Ok(self)
    }
}

impl IntoUrl for String {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(&self)
    }
}

impl<'a> IntoUrl for &'a str {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self)
    }
}

#[cfg(feature = "url1")]
impl IntoUrl for Url1 {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self.as_str())
    }
}

/// BitBar only supports up to five parameters for `bash=` commands (see <https://github.com/matryer/bitbar/issues/490>).
#[derive(Debug)]
pub enum Params {
    /// Just a command, no arguments.
    Zero([String; 1]),
    /// A command and 1 argument.
    One([String; 2]),
    /// A command and 2 arguments.
    Two([String; 3]),
    /// A command and 3 arguments.
    Three([String; 4]),
    /// A command and 4 arguments.
    Four([String; 5]),
    /// A command and 5 arguments.
    Five([String; 6])
}

impl Params {
    /// Iterates over the command and any arguments in order.
    pub fn iter(&self) -> impl Iterator<Item = &String> {
        match self {
            Params::Zero(a) => a.iter(),
            Params::One(a) => a.iter(),
            Params::Two(a) => a.iter(),
            Params::Three(a) => a.iter(),
            Params::Four(a) => a.iter(),
            Params::Five(a) => a.iter(),
        }
    }
}

macro_rules! params_from {
    ($n:literal, $variant:ident, $($elt:ident: $t:ident),+) => {
        impl<T: ToString> From<[T; $n]> for Params {
            fn from([$($elt),+]: [T; $n]) -> Params {
                Params::$variant([$($elt.to_string()),+])
            }
        }

        impl<$($t: ToString),+> From<($($t,)+)> for Params {
            fn from(($($elt,)+): ($($t,)+)) -> Params {
                Params::$variant([$($elt.to_string()),+])
            }
        }
    };
}

params_from!(1, Zero, cmd: A);
params_from!(2, One, cmd: A, param1: B);
params_from!(3, Two, cmd: A, param1: B, param2: C);
params_from!(4, Three, cmd: A, param1: B, param2: C, param3: D);
params_from!(5, Four, cmd: A, param1: B, param2: C, param3: D, param4: E);
params_from!(6, Five, cmd: A, param1: B, param2: C, param3: D, param4: E, param5: F);

impl<'a, T: ToString> TryFrom<&'a [T]> for Params {
    type Error = &'a [T];

    fn try_from(slice: &[T]) -> Result<Params, &[T]> {
        match slice {
            [cmd] => Ok(Params::Zero([cmd.to_string()])),
            [cmd, param1] => Ok(Params::One([cmd.to_string(), param1.to_string()])),
            [cmd, param1, param2] => Ok(Params::Two([cmd.to_string(), param1.to_string(), param2.to_string()])),
            [cmd, param1, param2, param3] => Ok(Params::Three([cmd.to_string(), param1.to_string(), param2.to_string(), param3.to_string()])),
            [cmd, param1, param2, param3, param4] => Ok(Params::Four([cmd.to_string(), param1.to_string(), param2.to_string(), param3.to_string(), param4.to_string()])),
            [cmd, param1, param2, param3, param4, param5] => Ok(Params::Five([cmd.to_string(), param1.to_string(), param2.to_string(), param3.to_string(), param4.to_string(), param5.to_string()])),
            slice => Err(slice)
        }
    }
}

impl<T: ToString> TryFrom<Vec<T>> for Params {
    type Error = Vec<T>;

    fn try_from(mut v: Vec<T>) -> Result<Params, Vec<T>> {
        match v.len() {
            1 => Ok(Params::Zero([v.remove(0).to_string()])),
            2 => Ok(Params::One([v.remove(0).to_string(), v.remove(0).to_string()])),
            3 => Ok(Params::Two([v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string()])),
            4 => Ok(Params::Three([v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string()])),
            5 => Ok(Params::Four([v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string()])),
            6 => Ok(Params::Five([v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string(), v.remove(0).to_string()])),
            _ => Err(v)
        }
    }
}

/// Used by `ContentItem::command`.
///
/// A `Command` contains the `Params`, which includes the actual command (called `bash=` by BitBar) and its parameters, and the value of `terminal=`.
///
/// It is usually constructed via conversion, unless `terminal=true` is required.
///
/// **Note:** Unlike BitBar's default of `true`, `Command` assumes a default of `terminal=false`.
#[derive(Debug)]
pub struct Command {
    params: Params,
    terminal: bool
}

impl Command {
    /// Creates a `Command` with the `terminal=` value set to `true`.
    pub fn terminal(args: impl Into<Params>) -> Command {
        Command {
            params: args.into(),
            terminal: true
        }
    }

    /// Attempts to construct a `Command` with `terminal=` set to `false` from the given arguments.
    ///
    /// This is not a `TryFrom` implementation due to a limitation in Rust.
    pub fn try_from<P: TryInto<Params>>(args: P) -> Result<Command, P::Error> {
        Ok(Command {
            params: args.try_into()?,
            terminal: false
        })
    }

    /// Same as `Command::terminal` but for types that might not convert to `Params`.
    pub fn try_terminal<P: TryInto<Params>>(args: P) -> Result<Command, P::Error> {
        Ok(Command {
            params: args.try_into()?,
            terminal: true
        })
    }
}

/// Converts an array containing a command string and 0–5 parameters to a command argument vector. The `terminal=` value will be `false`.
impl<P: Into<Params>> From<P> for Command {
    fn from(args: P) -> Command {
        Command {
            params: args.into(),
            terminal: false
        }
    }
}

/// Used by `ContentItem::image` and `ContentItem::template_image`.
#[derive(Debug)]
pub struct Image {
    /// The base64-encoded image data.
    pub base64_data: String,
    /// If this is `true`, the image will be used with BitBar's `templateImage=` instead of `image=`.
    pub is_template: bool
}

impl Image {
    /// Constructs a template image, even if the `TryInto` implementation would otherwise construct a non-template image.
    pub fn template<T: TryInto<Image>>(img: T) -> Result<Image, T::Error> {
        let mut result = img.try_into()?;
        result.is_template = true;
        Ok(result)
    }
}

/// Converts already-encoded base64 data to a non-template image.
impl From<String> for Image {
    fn from(base64_data: String) -> Image {
        Image {
            base64_data,
            is_template: false
        }
    }
}

/// Converts a PNG file to a non-template image.
#[cfg(feature = "base64")]
impl From<Vec<u8>> for Image {
    fn from(input: Vec<u8>) -> Image {
        Image {
            base64_data: base64::encode(&input),
            is_template: false
        }
    }
}

/// Converts a PNG file to a non-template image.
#[cfg(feature = "base64")]
impl<T: ?Sized + AsRef<[u8]>> From<&T> for Image {
    fn from(input: &T) -> Image {
        Image {
            base64_data: base64::encode(input),
            is_template: false
        }
    }
}

#[cfg(all(feature = "base64", feature = "image"))]
impl TryFrom<DynamicImage> for Image {
    type Error = ImageError;

    fn try_from(img: DynamicImage) -> ImageResult<Image> {
        let mut buf = Vec::default();
        img.write_to(&mut buf, PNG)?;
        Ok(Image::from(&buf))
    }
}

/// A menu item that's not a separator.
#[derive(Debug, Default)]
pub struct ContentItem {
    /// This menu item's main content text.
    ///
    /// Any `|` in the text will be displayed as `¦`, and any newlines will be displayed as spaces.
    pub text: String,
    /// This menu item's alternate-mode menu item or submenu.
    pub extra: Option<Extra>,
    /// Corresponds to BitBar's `href=` parameter.
    pub href: Option<Url>,
    /// Corresponds to BitBar's `color=` parameter.
    pub color: Option<Color>,
    /// Corresponds to BitBar's `font=` parameter.
    pub font: Option<String>,
    /// Corresponds to BitBar's `bash=`, `terminal=`, `param1=`, etc. parameters.
    pub command: Option<Command>,
    /// Corresponds to BitBar's `refresh=` parameter.
    pub refresh: bool,
    /// Corresponds to BitBar's `image=` or `templateImage=` parameter.
    pub image: Option<Image>
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
        self.extra = Some(Extra::Submenu(Menu::from_iter(items)));
        self
    }

    /// Adds a clickable link to this menu item.
    pub fn href(mut self, href: impl IntoUrl) -> Result<Self, url::ParseError> {
        self.href = Some(href.into_url()?);
        Ok(self)
    }

    /// Sets this menu item's text color. Alpha channel is ignored.
    pub fn color(mut self, color: impl IntoColor) -> Result<Self, ColorParseError> {
        self.color = Some(color.into_color()?);
        Ok(self)
    }

    /// Sets this menu item's text font.
    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = Some(font.to_string());
        self
    }

    /// Make this menu item run the given command when clicked.
    pub fn command(mut self, cmd: impl Into<Command>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    /// Causes the BitBar plugin to be refreshed when this menu item is clicked.
    pub fn refresh(mut self) -> Self {
        self.refresh = true;
        self
    }

    /// Adds an alternate menu item, which is shown instead of this one as long as the option key ⌥ is held.
    pub fn alt(mut self, alt: impl Into<ContentItem>) -> Self {
        self.extra = Some(Extra::Alternate(Box::new(alt.into())));
        self
    }

    /// Adds a template image to this menu item.
    pub fn template_image<T: TryInto<Image>>(mut self, img: T) -> Result<Self, T::Error> {
        self.image = Some(Image::template(img)?);
        Ok(self)
    }

    /// Adds an image to this menu item. The image will not be considered a template image unless specified as such by the `img` parameter.
    pub fn image<T: TryInto<Image>>(mut self, img: T) -> Result<Self, T::Error> {
        self.image = Some(img.try_into()?);
        Ok(self)
    }

    fn render(&self, f: &mut fmt::Formatter<'_>, is_alt: bool) -> fmt::Result {
        // main text
        write!(f, "{}", self.text.replace('|', "¦").replace('\n', " "))?;
        // parameters
        let mut rendered_params = BTreeMap::default();
        if let Some(ref href) = self.href {
            rendered_params.insert("href".into(), href.to_string());
        }
        if let Some(ref color) = self.color {
            rendered_params.insert("color".into(), format!("#{:02x}{:02x}{:02x}", color.r, color.g, color.b));
        }
        if let Some(ref font) = self.font {
            rendered_params.insert("font".into(), font.clone());
        }
        if let Some(ref cmd) = self.command {
            for (i, param) in cmd.params.iter().enumerate() {
                rendered_params.insert(if i == 0 { "bash".into() } else { format!("param{}", i) }, param.clone());
            }
            if !cmd.terminal {
                rendered_params.insert("terminal".into(), "false".into());
            }
        }
        if self.refresh {
            rendered_params.insert("refresh".into(), "true".into());
        }
        if is_alt {
            rendered_params.insert("alternate".into(), "true".into());
        }
        if let Some(ref img) = self.image {
            rendered_params.insert(if img.is_template { "templateImage" } else { "image" }.into(), img.base64_data.clone());
        }
        if !rendered_params.is_empty() {
            write!(f, " |")?;
            for (name, value) in rendered_params {
                let quoted_value = if value.contains(' ') {
                    format!("\"{}\"", value)
                } else {
                    value
                }; //TODO check for double quotes in value, fall back to single quotes? (test if BitBar supports these first)
                write!(f, " {}={}", name, quoted_value)?;
            }
        }
        writeln!(f)?;
        // additional items
        match &self.extra {
            Some(Extra::Alternate(ref alt)) => { alt.render(f, true)?; }
            Some(Extra::Submenu(ref sub)) => {
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
