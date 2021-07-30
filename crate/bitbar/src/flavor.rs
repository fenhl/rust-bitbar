//! Features specific to individual BitBar implementations (e.g. [SwiftBar](https://swiftbar.app/))

use {
    std::{
        env,
        fmt,
        sync::Arc,
    },
    semver::Version,
    crate::{
        Menu,
        MenuItem,
    },
};

#[derive(Debug, Clone, Copy)]
/// A BitBar implementation.
pub enum Flavor {
    /// The original, now discontinued implementation, with just the base features. This is also returned if a plugin is run on its own.
    BitBar,
    /// [SwiftBar](https://swiftbar.app/)
    SwiftBar(SwiftBar),
    //TODO xbar support, blocked on https://github.com/matryer/xbar/issues/753
    //TODO argos (https://github.com/p-e-w/argos) support?
}

impl Flavor {
    /// Checks which of the supported BitBar implementations the plugin is currently running on,
    /// returning a handle allowing use of implementation-specific features, if any are supported.
    /// Any unsupported implementation will be reported as `BitBar`.
    pub fn check() -> Flavor {
        if let Some(swiftbar) = SwiftBar::check() {
            Flavor::SwiftBar(swiftbar)
        } else {
            Flavor::BitBar
        }
    }
}

impl fmt::Display for Flavor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Flavor::SwiftBar(_) => write!(f, "SwiftBar"),
            Flavor::BitBar => write!(f, "BitBar"),
        }
    }
}

/// A type-safe handle for [SwiftBar](https://swiftbar.app/)-specific features.
#[derive(Debug, Clone, Copy)]
pub struct SwiftBar(()); // `()` field to make sure the type can't be instantiated outside of this module

impl SwiftBar {
    /// Checks whether the plugins is running in SwiftBar by checking environment variables.
    /// If it does, returns a handle allowing use of SwiftBar-specific features.
    pub fn check() -> Option<Self> {
        env::var_os("SWIFTBAR").map(|_| Self(()))
    }

    /// Returns a handle allowing use of SwiftBar-specific features **without checking whether the plugin is actually running inside SwiftBar**.
    /// If the plugin is actually running in a different implementation, this may lead to incorrect behavior.
    #[cfg(feature = "assume-flavor")]
    #[cfg_attr(docsrs, doc(cfg(feature = "assume-flavor")))]
    pub fn assume() -> Self { SwiftBar(()) }

    /// Returns the SwiftBar version on which the plugin is running by checking environment variables.
    pub fn running_version(&self) -> Result<Version, VersionCheckError> {
        Ok(env::var("SWIFTBAR_VERSION")?.parse()?)
    }
}

/// An error that can occur when checking the running SwiftBar version.
#[derive(Debug, Clone)]
pub enum VersionCheckError {
    /// The `SWIFTBAR_VERSION` environment variable was unset or not valid UTF-8
    Env(env::VarError),
    /// The `SWIFTBAR_VERSION` environment variable was not a valid semantic version
    Parse(Arc<semver::Error>),
}

impl From<env::VarError> for VersionCheckError {
    fn from(e: env::VarError) -> VersionCheckError {
        VersionCheckError::Env(e)
    }
}

impl From<semver::Error> for VersionCheckError {
    fn from(e: semver::Error) -> VersionCheckError {
        VersionCheckError::Parse(Arc::new(e))
    }
}

impl From<VersionCheckError> for Menu {
    fn from(e: VersionCheckError) -> Menu {
        let mut menu = vec![MenuItem::new(format!("Error checking running {} version", Flavor::check()))];
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
