//! Parameters for modifying the appearance or behavior of [`ContentItem`]s.

use {
    std::{
        convert::{
            TryFrom,
            TryInto,
        },
        fmt,
        str::FromStr,
    },
    css_color_parser::ColorParseError,
    url::Url,
    crate::{
        ContentItem,
        Menu,
    },
};
#[cfg(feature = "url1")] use url1::Url as Url1;
#[cfg(all(feature = "base64", feature = "image"))] use {
    image::{
        DynamicImage,
        ImageError,
        ImageOutputFormat::PNG,
        ImageResult,
    },
};

/// Used in [`ContentItem::color`](ContentItem::color()).
///
/// Construct via [`Into`] or [`TryInto`](std::convert::TryInto) implementations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub(crate) light: css_color_parser::Color,
    /// SwiftBar only: separate color for dark system theme. If `None`, use `light`.
    pub(crate) dark: Option<css_color_parser::Color>,
}

impl From<css_color_parser::Color> for Color {
    fn from(light: css_color_parser::Color) -> Color {
        Color { light, dark: None }
    }
}

impl FromStr for Color {
    type Err = ColorParseError;

    fn from_str(s: &str) -> Result<Color, ColorParseError> {
        Ok(Color {
            light: s.parse()?,
            dark: None,
        })
    }
}

impl<'a> TryFrom<&'a str> for Color {
    type Error = ColorParseError;

    fn try_from(s: &str) -> Result<Color, ColorParseError> {
        s.parse()
    }
}

#[cfg(feature = "css-colors")]
macro_rules! css_color_try_into_color {
    ($t:ty) => {
        #[cfg_attr(docsrs, doc(cfg(feature = "css-colors")))]
        impl TryFrom<$t> for Color {
            type Error = ColorParseError;

            fn try_from(color: $t) -> Result<Color, ColorParseError> {
                Ok(Color {
                    light: color.to_string().parse()?,
                    dark: None,
                })
            }
        }
    };
}

#[cfg(feature = "css-colors")] css_color_try_into_color!(css_colors::RGB);
#[cfg(feature = "css-colors")] css_color_try_into_color!(css_colors::RGBA);
#[cfg(feature = "css-colors")] css_color_try_into_color!(css_colors::HSL);
#[cfg(feature = "css-colors")] css_color_try_into_color!(css_colors::HSLA);

#[cfg(feature = "serenity")]
#[cfg_attr(docsrs, doc(cfg(feature = "serenity")))]
impl From<serenity::utils::Colour> for Color {
    fn from(c: serenity::utils::Colour) -> Color {
        Color {
            light: css_color_parser::Color {
                r: c.r(),
                g: c.g(),
                b: c.b(),
                a: 1.0,
            },
            dark: None,
        }
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.light.r, self.light.g, self.light.b)?;
        if let Some(dark) = self.dark {
            write!(f, ",#{:02x}{:02x}{:02x}", dark.r, dark.g, dark.b)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
/// A menu item's alternate mode or submenu.
pub enum Extra {
    /// A menu item's alternate mode, shown when <key>⌥</key> is held.
    Alternate(Box<ContentItem>), //TODO make sure alts don't have submenus
    /// A submenu.
    Submenu(Menu),
}

/// Used by [`ContentItem::href`](ContentItem::href()).
pub trait IntoUrl {
    /// Converts `self` into a [`Url`].
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
#[cfg_attr(docsrs, doc(cfg(feature = "url1")))]
impl IntoUrl for Url1 {
    fn into_url(self) -> Result<Url, url::ParseError> {
        Url::parse(self.as_str())
    }
}

/// BitBar only supports up to five parameters for `bash=` commands (see <https://github.com/matryer/bitbar/issues/490>).
#[derive(Debug)]
pub struct Params {
    pub(crate) cmd: String,
    pub(crate) params: Vec<String>,
}

impl Params {
    #[doc(hidden)] // used in proc macro
    pub fn new(cmd: String, params: Vec<String>) -> Self {
        Self { cmd, params }
    }
}

macro_rules! params_from {
    ($n:literal$(, $elt:ident: $t:ident)*) => {
        impl<T: ToString> From<[T; $n]> for Params {
            fn from([cmd, $($elt),*]: [T; $n]) -> Params {
                Params {
                    cmd: cmd.to_string(),
                    params: vec![$($elt.to_string()),*],
                }
            }
        }

        impl<Cmd: ToString, $($t: ToString),*> From<(Cmd, $($t),*)> for Params {
            fn from((cmd, $($elt),*): (Cmd, $($t),*)) -> Params {
                Params {
                    cmd: cmd.to_string(),
                    params: vec![$($elt.to_string()),*],
                }
            }
        }
    };
}

params_from!(1);
params_from!(2, param1: A);
params_from!(3, param1: A, param2: B);
params_from!(4, param1: A, param2: B, param3: C);
params_from!(5, param1: A, param2: B, param3: C, param4: D);
params_from!(6, param1: A, param2: B, param3: C, param4: D, param5: E);

impl<'a, T: ToString> TryFrom<&'a [T]> for Params {
    type Error = &'a [T];

    fn try_from(slice: &[T]) -> Result<Params, &[T]> {
        match slice {
            [cmd] => Ok(Params { cmd: cmd.to_string(), params: Vec::default() }),
            [cmd, param1] => Ok(Params { cmd: cmd.to_string(), params: vec![param1.to_string()] }),
            [cmd, param1, param2] => Ok(Params { cmd: cmd.to_string(), params: vec![param1.to_string(), param2.to_string()] }),
            [cmd, param1, param2, param3] => Ok(Params { cmd: cmd.to_string(), params: vec![param1.to_string(), param2.to_string(), param3.to_string()] }),
            [cmd, param1, param2, param3, param4] => Ok(Params { cmd: cmd.to_string(), params: vec![param1.to_string(), param2.to_string(), param3.to_string(), param4.to_string()] }),
            [cmd, param1, param2, param3, param4, param5] => Ok(Params { cmd: cmd.to_string(), params: vec![param1.to_string(), param2.to_string(), param3.to_string(), param4.to_string(), param5.to_string()] }),
            slice => Err(slice),
        }
    }
}

impl<T: ToString> TryFrom<Vec<T>> for Params {
    type Error = Vec<T>;

    fn try_from(mut v: Vec<T>) -> Result<Params, Vec<T>> {
        match v.len() {
            1..=6 => Ok(Params {
                cmd: v.remove(0).to_string(),
                params: v.into_iter().map(|x| x.to_string()).collect(),
            }),
            _ => Err(v),
        }
    }
}

/// Used by [`ContentItem::command`](ContentItem::command()).
///
/// A `Command` contains the [`Params`], which includes the actual command (called `bash=` by BitBar) and its parameters, and the value of `terminal=`.
///
/// It is usually constructed via conversion, unless `terminal=true` is required.
///
/// **Note:** Unlike BitBar's default of `true`, `Command` assumes a default of `terminal=false`.
#[derive(Debug)]
pub struct Command {
    pub(crate) params: Params,
    pub(crate) terminal: bool,
}

impl Command {
    /// Creates a `Command` with the `terminal=` value set to `true`.
    pub fn terminal(args: impl Into<Params>) -> Command {
        Command {
            params: args.into(),
            terminal: true,
        }
    }

    /// Attempts to construct a `Command` with `terminal=` set to `false` from the given arguments.
    ///
    /// This is not a `TryFrom` implementation due to a limitation in Rust.
    pub fn try_from<P: TryInto<Params>>(args: P) -> Result<Command, P::Error> {
        Ok(Command {
            params: args.try_into()?,
            terminal: false,
        })
    }

    /// Same as `Command::terminal` but for types that might not convert to `Params`.
    pub fn try_terminal<P: TryInto<Params>>(args: P) -> Result<Command, P::Error> {
        Ok(Command {
            params: args.try_into()?,
            terminal: true,
        })
    }
}

/// Converts an array containing a command string and 0–5 parameters to a command argument vector. The `terminal=` value will be `false`.
impl<P: Into<Params>> From<P> for Command {
    fn from(args: P) -> Command {
        Command {
            params: args.into(),
            terminal: false,
        }
    }
}

/// Used by `ContentItem::image` and `ContentItem::template_image`.
#[derive(Debug, Clone)]
pub struct Image {
    /// The base64-encoded image data.
    pub base64_data: String,
    /// If this is `true`, the image will be used with BitBar's `templateImage=` instead of `image=`.
    pub is_template: bool,
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
            is_template: false,
        }
    }
}

/// Converts a PNG file to a non-template image.
#[cfg(feature = "base64")]
#[cfg_attr(docsrs, doc(cfg(feature = "base64")))]
impl From<Vec<u8>> for Image {
    fn from(input: Vec<u8>) -> Image {
        Image {
            base64_data: base64::encode(&input),
            is_template: false,
        }
    }
}

/// Converts a PNG file to a non-template image.
#[cfg(feature = "base64")]
#[cfg_attr(docsrs, doc(cfg(feature = "base64")))]
impl<T: ?Sized + AsRef<[u8]>> From<&T> for Image {
    fn from(input: &T) -> Image {
        Image {
            base64_data: base64::encode(input),
            is_template: false,
        }
    }
}

#[cfg(all(feature = "base64", feature = "image"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "base64", feature = "image"))))]
impl TryFrom<DynamicImage> for Image {
    type Error = ImageError;

    fn try_from(img: DynamicImage) -> ImageResult<Image> {
        let mut buf = Vec::default();
        img.write_to(&mut buf, PNG)?;
        Ok(Image::from(&buf))
    }
}
