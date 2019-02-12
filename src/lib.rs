use std::{
    collections::BTreeMap,
    fmt,
    iter::FromIterator
};
use css_color_parser::{
    Color,
    ColorParseError
};
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

macro_rules! impl_into_color_for_css_color {
    ($t:ty) => {
        impl IntoColor for $t {
            fn into_color(self) -> Result<Color, ColorParseError> {
                Ok(self.to_string().parse()?)
            }
        }
    };
}

impl_into_color_for_css_color!(css_colors::RGB);
impl_into_color_for_css_color!(css_colors::RGBA);
impl_into_color_for_css_color!(css_colors::HSL);
impl_into_color_for_css_color!(css_colors::HSLA);

#[derive(Debug)]
pub struct Command {
    pub args: Vec<String>,
    pub terminal: bool
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
        write!(f, "{}", self.text)?; //TODO escape pipes
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
            for (i, arg) in cmd.args.iter().enumerate() {
                rendered_params.insert(if i == 0 { "bash".into() } else { format!("param{}", i) }, arg.clone());
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
                write!(f, " {}={}", name, value)?; //TODO quoting
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
