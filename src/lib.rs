use std::{
    collections::BTreeMap,
    fmt
};

#[derive(Debug)]
pub enum Extra {
    Alternate(Box<ContentItem>), //TODO make sure alts don't have submenus
    Submenu(Menu)
}

#[derive(Debug)]
pub struct Command {
    pub args: Vec<String>,
    pub terminal: bool
}

#[derive(Debug, Default)]
pub struct ContentItem {
    pub text: String,
    pub extra: Option<Extra>,
    pub font: Option<String>,
    pub command: Option<Command>,
    pub refresh: bool
}

impl ContentItem {
    fn render(&self, f: &mut fmt::Formatter, is_alt: bool) -> fmt::Result {
        // main text
        write!(f, "{}", self.text)?; //TODO escape pipes
        // parameters
        let mut rendered_params = BTreeMap::default();
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
        if let Some(ref font) = self.font {
            rendered_params.insert("font".into(), font.clone());
        }
        if is_alt {
            rendered_params.insert("alternate".into(), "true".into());
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

impl Default for MenuItem {
    fn default() -> MenuItem {
        MenuItem::Content(ContentItem::default())
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

#[derive(Debug)]
pub struct Menu(pub Vec<MenuItem>);

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
