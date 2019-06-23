//! This is `bitbar`, a library crate which includes helpers for writing [BitBar](https://getbitbar.com/) plugins in Rust. The main feature is the `Menu` type whose `Display` implementation generates output that conforms to the [BitBar plugin API](https://github.com/matryer/bitbar#plugin-api).
//!
//! # Features
//!
//! The following features can be enabled via Cargo:
//!
//! * `css-colors`: Adds a dependency to the [`css-colors`](https://crates.io/crates/css-colors) crate and implements `IntoColor` for its color types `RGB`, `RGBA`, `HSL`, and `HSLA`.
//! * `serenity`: Adds a dependency to the [`serenity`](https://crates.io/crates/serenity) crate and implements `IntoColor` for its `Colour` type.

use std::{
    collections::BTreeMap,
    fmt,
    iter::FromIterator
};
use css_color_parser::{
    Color,
    ColorParseError
};
use derive_more::From;
use url::Url;

#[derive(Debug)]
pub enum Extra {
    Alternate(Box<ContentItem>), //TODO make sure alts don't have submenus
    Submenu(Menu)
}

pub trait IntoColor {
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

/// BitBar only supports up to five parameters for `bash=` commands (see <https://github.com/matryer/bitbar/issues/490>).
#[derive(Debug, From)]
pub enum Params {
    Zero([String; 1]),
    One([String; 2]),
    Two([String; 3]),
    Three([String; 4]),
    Four([String; 5]),
    Five([String; 6])
}

impl Params {
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

#[derive(Debug)]
pub struct Command {
    params: Params,
    terminal: bool
}

impl Command {
    /// Creates a `Command` with the `terminal` value set to `true`.
    pub fn terminal(args: impl Into<Params>) -> Command {
        Command {
            params: args.into(),
            terminal: true
        }
    }
}

/// Converts an array containing a command string and 0–5 parameters to a command argument vector. The `terminal` value will be `false`.
impl<P: Into<Params>> From<P> for Command {
    fn from(params: P) -> Command {
        Command {
            params: params.into(),
            terminal: false
        }
    }
}

#[derive(Debug)]
pub struct Image {
    pub base64_data: String,
    pub is_template: bool
}

#[derive(Debug, Default)]
pub struct ContentItem {
    pub text: String,
    pub extra: Option<Extra>,
    pub href: Option<Url>,
    pub color: Option<Color>,
    pub font: Option<String>,
    pub command: Option<Command>,
    pub refresh: bool,
    pub image: Option<Image>
}

impl ContentItem {
    pub fn new(text: impl ToString) -> ContentItem {
        ContentItem {
            text: text.to_string(),
            ..ContentItem::default()
        }
    }

    pub fn sub(mut self, items: impl IntoIterator<Item = MenuItem>) -> Self {
        self.extra = Some(Extra::Submenu(Menu::from_iter(items)));
        self
    }

    pub fn href(mut self, href: Url) -> Self {
        self.href = Some(href);
        self
    }

    /// Sets the menu item's text color. Alpha channel is ignored.
    pub fn color(mut self, color: impl IntoColor) -> Result<Self, ColorParseError> {
        self.color = Some(color.into_color()?);
        Ok(self)
    }

    pub fn font(mut self, font: impl ToString) -> Self {
        self.font = Some(font.to_string());
        self
    }

    pub fn command(mut self, cmd: impl Into<Command>) -> Self {
        self.command = Some(cmd.into());
        self
    }

    pub fn refresh(mut self) -> Self {
        self.refresh = true;
        self
    }

    pub fn alt(mut self, alt: impl Into<ContentItem>) -> Self {
        self.extra = Some(Extra::Alternate(Box::new(alt.into())));
        self
    }

    pub fn template_image(mut self, img: impl ToString) -> Self { //TODO support image types
        self.image = Some(Image {
            base64_data: img.to_string(),
            is_template: true
        });
        self
    }

    pub fn image(mut self, img: impl ToString) -> Self { //TODO support image types
        self.image = Some(Image {
            base64_data: img.to_string(),
            is_template: false
        });
        self
    }

    fn render(&self, f: &mut fmt::Formatter, is_alt: bool) -> fmt::Result {
        // main text
        write!(f, "{}", self.text)?; //TODO escape pipes and newlines
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.render(f, false)
    }
}

#[derive(Debug)]
pub enum MenuItem {
    Content(ContentItem),
    Sep
}

impl MenuItem {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            MenuItem::Content(content) => write!(f, "{}", content),
            MenuItem::Sep => writeln!(f, "---")
        }
    }
}

#[derive(Debug, Default)]
pub struct Menu(pub Vec<MenuItem>);

impl<A: Into<MenuItem>> FromIterator<A> for Menu {
    fn from_iter<T: IntoIterator<Item = A>>(iter: T) -> Menu {
        Menu(iter.into_iter().map(Into::into).collect())
    }
}

/// This provides the main functionality of this crate: rendering a BitBar plugin.
///
/// Note that the output this generates already includes a trailing newline, so it should be used with `print!` instead of `println!`.
impl fmt::Display for Menu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for menu_item in &self.0 {
            write!(f, "{}", menu_item)?;
        }
        Ok(())
    }
}
