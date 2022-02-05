//! Features specific to [SwiftBar](https://swiftbar.app/)

use {
    std::{
        borrow::Cow,
        collections::BTreeMap,
        convert::TryInto,
        env,
        io,
        iter,
        path::Path,
        sync::Arc,
    },
    open::that as open,
    semver::Version,
    thiserror::Error,
    url::Url,
    crate::{
        ContentItem,
        MainOutput,
        Menu,
        MenuItem,
        attr::{
            Color,
            Command,
            Image,
            IntoUrl,
            Params,
        },
    },
};
#[cfg(feature = "assume-flavor")] use static_assertions::const_assert;
#[cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))] use {
    std::pin::Pin,
    futures::{
        future::Future,
        stream::StreamExt as _,
    },
    crate::AsyncMainOutput,
};

/// The highest build number checked for conditional features.
#[cfg(feature = "assume-flavor")] const MAX_BUILD: usize = 402;

macro_rules! build_ge {
    ($swiftbar:expr, $build:expr) => {{
        #[cfg(feature = "assume-flavor")] const_assert!($build <= MAX_BUILD);
        $swiftbar.build >= $build
    }};
}

/// A type-safe handle for [SwiftBar](https://swiftbar.app/)-specific features.
///
/// Some SwiftBar-specific features are currently unsupported:
///
/// * [Script metadata](https://github.com/swiftbar/SwiftBar#script-metadata) is unsupported since `cargo` does not support adding metadata to binaries it produces. You will have to [add any metadata via `xattr`](https://github.com/swiftbar/SwiftBar#metadata-for-binary-plugins).
#[derive(Debug, Clone, Copy)]
pub struct SwiftBar {
    build: usize,
}

impl SwiftBar {
    /// Checks whether the plugins is running in SwiftBar by checking environment variables.
    /// If it does, returns a handle allowing use of SwiftBar-specific features.
    pub fn check() -> Option<Self> {
        Some(Self {
            build: env::var("SWIFTBAR_BUILD").ok()?.parse().ok()?,
        })
    }

    #[cfg(feature = "assume-flavor")]
    #[cfg_attr(docsrs, doc(cfg(feature = "assume-flavor")))]
    /// Returns a handle allowing use of SwiftBar-specific features **without checking whether the plugin is actually running inside SwiftBar**.
    /// If the plugin is actually running in a different implementation or an outdated version of SwiftBar, this may lead to incorrect behavior.
    pub fn assume() -> Self {
        Self {
            build: MAX_BUILD,
        }
    }

    /// The name of the plugin, including refresh time and file extension, as used in [`swiftbar:` URLs](https://github.com/swiftbar/SwiftBar#url-scheme).
    pub fn plugin_name(&self) -> Result<String, PluginNameError> {
        Ok(Path::new(&env::var_os("SWIFTBAR_PLUGIN_PATH").ok_or(PluginNameError::Env)?)
            .file_name().ok_or(PluginNameError::NoFileName)?
            .to_str().ok_or(PluginNameError::NonUtf8FileName)?
            .to_owned()
        )
    }

    /// Returns the SwiftBar version on which the plugin is running by checking environment variables.
    pub fn running_version(&self) -> Result<Version, VersionCheckError> {
        Ok(env::var("SWIFTBAR_VERSION")?.parse()?)
    }

    /// Unlike BitBar, SwiftBar supports more than 5 parameters for `bash=` commands.
    pub fn command(&self, cmd: impl IntoParams) -> Params {
        cmd.into_params(self)
    }

    /// Returns a [`Color`](crate::param::Color) that renders differently depending on whether the system is in dark mode.
    pub fn themed_color(&self, light: Color, dark: Color) -> Color {
        Color {
            light: light.light,
            dark: Some(dark.dark.unwrap_or(dark.light)),
        }
    }

    /// Adds a [SF Symbols](https://developer.apple.com/sf-symbols/) image to a menu item.
    pub fn sf_image(&self, item: &mut ContentItem, image: impl ToString) {
        Attrs::for_item(item).sf_image = Some(image.to_string());
    }
}

/// A type that can be used as `bash=` command parameters for SwiftBar, which unlike BitBar supports more than five parameters.
pub trait IntoParams {
    /// Converts this value into command parameters.
    ///
    /// Equivalent to `swiftbar.command(self)`.
    fn into_params(self, swiftbar: &SwiftBar) -> Params;
}

impl IntoParams for Params {
    fn into_params(self, _: &SwiftBar) -> Params {
        self
    }
}

macro_rules! impl_into_params {
    ($n:literal$(, $elt:ident: $t:ident)*) => {
        impl<T: ToString> IntoParams for [T; $n] {
            fn into_params(self, _: &SwiftBar) -> Params {
                let [cmd, $($elt),*] = self;
                Params {
                    cmd: cmd.to_string(),
                    params: vec![$($elt.to_string()),*],
                }
            }
        }

        impl<Cmd: ToString, $($t: ToString),*> IntoParams for (Cmd, $($t),*) {
            fn into_params(self, _: &SwiftBar) -> Params {
                let (cmd, $($elt),*) = self;
                Params {
                    cmd: cmd.to_string(),
                    params: vec![$($elt.to_string()),*],
                }
            }
        }
    };
}

impl_into_params!(1);
impl_into_params!(2, param1: A);
impl_into_params!(3, param1: A, param2: B);
impl_into_params!(4, param1: A, param2: B, param3: C);
impl_into_params!(5, param1: A, param2: B, param3: C, param4: D);
impl_into_params!(6, param1: A, param2: B, param3: C, param4: D, param5: E);
impl_into_params!(7, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F);
impl_into_params!(8, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G);
impl_into_params!(9, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H);
impl_into_params!(10, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I);
impl_into_params!(11, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J);
impl_into_params!(12, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J, param11: K);
impl_into_params!(13, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J, param11: K, param12: L);
impl_into_params!(14, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J, param11: K, param12: L, param13: M);
impl_into_params!(15, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J, param11: K, param12: L, param13: M, param14: N);
impl_into_params!(16, param1: A, param2: B, param3: C, param4: D, param5: E, param6: F, param7: G, param8: H, param9: I, param10: J, param11: K, param12: L, param13: M, param14: N, param15: O);

impl<'a, T: ToString> IntoParams for &'a [T] {
    /// # Panics
    ///
    /// If `self` is empty.
    fn into_params(self, _: &SwiftBar) -> Params {
        Params {
            cmd: self[0].to_string(),
            params: self[1..].iter().map(|param| param.to_string()).collect(),
        }
    }
}

impl<T: ToString> IntoParams for Vec<T> {
    /// # Panics
    ///
    /// If `self` is empty.
    fn into_params(mut self, _: &SwiftBar) -> Params {
        Params {
            cmd: self.remove(0).to_string(),
            params: self.into_iter().map(|param| param.to_string()).collect(),
        }
    }
}

/// Flavor-specific [`ContentItem`] attributes.
#[derive(Debug)]
pub struct Attrs {
    sf_image: Option<String>,
}

impl Attrs {
    fn for_item(item: &mut ContentItem) -> &mut Attrs {
        match item.flavor_attrs.get_or_insert(super::Attrs::SwiftBar(Attrs { sf_image: None })) {
            super::Attrs::SwiftBar(ref mut params) => params,
        }
    }

    pub(crate) fn render<'a>(&'a self, rendered_params: &mut BTreeMap<Cow<'a, str>, Cow<'a, str>>) {
        if let Some(ref sf_image) = self.sf_image {
            rendered_params.insert(Cow::Borrowed("sfimage"), Cow::Borrowed(sf_image));
        }
    }
}

/// An error that can occur when checking the running SwiftBar version.
#[derive(Debug, Error, Clone)]
pub enum VersionCheckError {
    /// The `SWIFTBAR_VERSION` environment variable was unset or not valid UTF-8
    #[error(transparent)] Env(#[from] env::VarError),
    /// The `SWIFTBAR_VERSION` environment variable was not a valid semantic version
    #[error(transparent)] Parse(Arc<semver::Error>),
}

impl From<semver::Error> for VersionCheckError {
    fn from(e: semver::Error) -> VersionCheckError {
        VersionCheckError::Parse(Arc::new(e))
    }
}

impl From<VersionCheckError> for Menu {
    fn from(e: VersionCheckError) -> Menu {
        let mut menu = vec![MenuItem::new("Error checking running SwiftBar version")];
        match e {
            VersionCheckError::Env(e) => menu.push(MenuItem::new(e)),
            VersionCheckError::Parse(e) => {
                menu.push(MenuItem::new(format!("error parsing version: {}", e)));
                menu.push(MenuItem::new(format!("{:?}", e)));
            }
        }
        Menu(menu)
    }
}

/// An error that can occur when checking the running SwiftBar plugin name.
#[derive(Debug, Error, Clone)]
pub enum PluginNameError {
    /// The `SWIFTBAR_PLUGIN_PATH` environment variable was unset
    #[error("missing `SWIFTBAR_PLUGIN_PATH` environment variable")]
    Env,
    /// The `SWIFTBAR_PLUGIN_PATH` environment variable did not end in a file name
    #[error("no filename in `SWIFTBAR_PLUGIN_PATH` environment variable")]
    NoFileName,
    /// The file name was not valid UTF-8
    #[error("plugin filename is not valid UTF-8")]
    NonUtf8FileName,
}

impl From<PluginNameError> for Menu {
    fn from(e: PluginNameError) -> Menu {
        Menu(vec![
            MenuItem::new("Error checking running SwiftBar plugin name"),
            MenuItem::new(e.to_string()),
        ])
    }
}

/// An error that can occur in [`Notification::command`].
#[derive(Debug, Error, Clone)]
pub enum NotificationCommandError<C: TryInto<Command>>
where C::Error: std::error::Error {
    /// Converting the parameter to a `Command` failed
    #[error(transparent)] Command(C::Error),
    /// Running commands on notification click is only supported on SwiftBar 1.4.3 beta 4 or newer
    #[error("running commands on notification click is only supported on SwiftBar 1.4.3 beta 4 or newer")]
    UnsupportedSwiftBarVersion,
}

impl<C: TryInto<Command>> From<NotificationCommandError<C>> for Menu
where C::Error: std::error::Error {
    fn from(e: NotificationCommandError<C>) -> Menu {
        let mut menu = vec![MenuItem::new("Error adding command to notification")];
        match e {
            NotificationCommandError::Command(e) => {
                menu.push(MenuItem::new(format!("error building command: {}", e)));
                menu.push(MenuItem::new(format!("{:?}", e)));
            }
            NotificationCommandError::UnsupportedSwiftBarVersion => menu.push(MenuItem::new("running commands on notification click is only supported on SwiftBar 1.4.3 beta 4 or newer")),
        }
        Menu(menu)
    }
}

/// A SwiftBar notification that can be opened as a URL.
pub struct Notification {
    swiftbar: SwiftBar,
    plugin_name: String,
    title: Option<String>,
    subtitle: Option<String>,
    body: Option<String>,
    href: Option<Url>,
    command: Option<Command>,
    silent: bool,
}

impl Notification {
    /// Creates a new notification with default options.
    ///
    /// Call methods on the returned instance to configure it.
    pub fn new(swiftbar: SwiftBar) -> Result<Self, PluginNameError> {
        Ok(Self {
            swiftbar,
            plugin_name: swiftbar.plugin_name()?,
            title: None,
            subtitle: None,
            body: None,
            href: None,
            command: None,
            silent: false,
        })
    }

    /// Sets the title for this notification.
    pub fn title(mut self, title: impl ToString) -> Self {
        self.title = Some(title.to_string());
        self
    }

    /// Sets the subtitle for this notification.
    pub fn subtitle(mut self, subtitle: impl ToString) -> Self {
        self.subtitle = Some(subtitle.to_string());
        self
    }

    /// Sets the text for this notification.
    pub fn body(mut self, body: impl ToString) -> Self {
        self.body = Some(body.to_string());
        self
    }

    /// Adds an URL that will be opened when this notification is clicked.
    pub fn href(mut self, href: impl IntoUrl) -> Result<Self, url::ParseError> {
        self.href = Some(href.into_url()?);
        Ok(self)
    }

    /// Makes this notification run the given command when clicked.
    pub fn command<C: TryInto<Command>>(mut self, cmd: C) -> Result<Self, NotificationCommandError<C>>
    where C::Error: std::error::Error {
        if build_ge!(self.swiftbar, 402) {
            self.command = Some(cmd.try_into().map_err(NotificationCommandError::Command)?);
            Ok(self)
        } else {
            Err(NotificationCommandError::UnsupportedSwiftBarVersion)
        }
    }

    /// Disables sound for this notification.
    pub fn silent(mut self) -> Self {
        self.silent = true;
        self
    }

    /// Displays this notification.
    pub fn send(&self) -> io::Result<()> {
        open(self.into_url().expect("failed to build SwiftBar notification URL").as_str())
    }
}

impl IntoUrl for Notification {
    fn into_url(self) -> Result<Url, url::ParseError> {
        (&self).into_url()
    }
}

impl<'a> IntoUrl for &'a Notification {
    fn into_url(self) -> Result<Url, url::ParseError> {
        let Notification { swiftbar: _, plugin_name, title, subtitle, body, command, href, silent } = self;
        Url::parse_with_params("swiftbar://notify", iter::once((Cow::Borrowed("plugin"), &**plugin_name))
            .chain(title.as_deref().map(|title| (Cow::Borrowed("title"), title)))
            .chain(subtitle.as_deref().map(|subtitle| (Cow::Borrowed("subtitle"), subtitle)))
            .chain(body.as_deref().map(|body| (Cow::Borrowed("body"), body)))
            .chain(command.iter().flat_map(|command| iter::once((Cow::Borrowed("bash"), &*command.params.cmd))
                .chain(command.params.params.iter().enumerate().map(|(n, arg)| (Cow::Owned(format!("param{}", n + 1)), &**arg)))
                .chain((!command.terminal).then(|| (Cow::Borrowed("terminal"), "false")))
            ))
            .chain(href.as_ref().map(|href| (Cow::Borrowed("href"), href.as_str())))
            .chain(silent.then(|| (Cow::Borrowed("silent"), "true")))
        )
    }
}

/// A type that [streams](https://github.com/swiftbar/SwiftBar#streamable) menus from an iterator.
///
/// Note that the following [plugin metadata](https://github.com/swiftbar/SwiftBar#script-metadata) items must be set for this to work:
/// * `<swiftbar.type>streamable</swiftbar.type>`
/// * `<swiftbar.useTrailingStreamSeparator>true</swiftbar.useTrailingStreamSeparator>`
///
/// The [`cargo-bitbar`](https://crates.io/crates/cargo-bitbar) crate can be used to add this metadata to the plugin. First, add this to your *workspace* manifest:
///
/// ```toml
/// [workspace.metadata.bitbar]
/// type = "streamable"
/// ```
///
/// Then, after building the plugin, run `cargo bitbar attr target/release/my-bitbar-plugin`.
pub struct BlockingStream<'a, I: MainOutput> {
    swiftbar: SwiftBar,
    inner: Box<dyn Iterator<Item = I> + 'a>,
}

impl<'a, I: MainOutput> BlockingStream<'a, I> {
    #[allow(missing_docs)]
    pub fn new(swiftbar: SwiftBar, iter: impl IntoIterator<Item = I> + 'a) -> Self {
        Self { swiftbar, inner: Box::new(iter.into_iter()) }
    }
}

impl<'a, I: MainOutput> MainOutput for BlockingStream<'a, I> {
    fn main_output(self, error_template_image: Option<Image>) {
        if build_ge!(self.swiftbar, 399) {
            for elt in self.inner {
                elt.main_output(error_template_image.clone());
                println!("~~~");
            }
        } else {
            for elt in self.inner {
                println!("~~~");
                elt.main_output(error_template_image.clone());
            }
        }
    }
}

#[cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))))]
/// A type that [streams](https://github.com/swiftbar/SwiftBar#streamable) menus from a stream (async iterator).
///
/// Note that the following [plugin metadata](https://github.com/swiftbar/SwiftBar#script-metadata) items must be set for this to work:
/// * `<swiftbar.type>streamable</swiftbar.type>`
/// * `<swiftbar.useTrailingStreamSeparator>true</swiftbar.useTrailingStreamSeparator>`
///
/// The [`cargo-bitbar`](https://crates.io/crates/cargo-bitbar) crate can be used to add this metadata to the plugin. First, add this to your *workspace* manifest:
///
/// ```toml
/// [workspace.metadata.bitbar]
/// type = "streamable"
/// ```
///
/// Then, after building the plugin, run `cargo bitbar attr target/release/my-bitbar-plugin`.
pub struct Stream<'a, I: AsyncMainOutput<'a> + 'a> {
    swiftbar: SwiftBar,
    inner: Pin<Box<dyn futures::stream::Stream<Item = I> + 'a>>,
}

#[cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))))]
impl<'a, I: AsyncMainOutput<'a> + 'a> Stream<'a, I> {
    #[allow(missing_docs)]
    pub fn new(swiftbar: SwiftBar, stream: impl futures::stream::Stream<Item = I> + 'a) -> Self {
        Self { swiftbar, inner: Box::pin(stream) }
    }
}

#[cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))]
#[cfg_attr(docsrs, doc(cfg(any(feature = "tokio", feature = "tokio02", feature = "tokio03"))))]
impl<'a, I: AsyncMainOutput<'a> + 'a> AsyncMainOutput<'a> for Stream<'a, I> {
    fn main_output(mut self, error_template_image: Option<Image>) -> Pin<Box<dyn Future<Output = ()> + 'a>> {
        if build_ge!(self.swiftbar, 399) {
            Box::pin(async move {
                while let Some(elt) = self.inner.next().await {
                    elt.main_output(error_template_image.clone()).await;
                    println!("~~~");
                }
            })
        } else {
            Box::pin(async move {
                while let Some(elt) = self.inner.next().await {
                    println!("~~~");
                    elt.main_output(error_template_image.clone()).await;
                }
            })
        }
    }
}
