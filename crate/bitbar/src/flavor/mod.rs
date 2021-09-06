//! Features specific to individual BitBar implementations (e.g. [SwiftBar](https://swiftbar.app/))

use std::{
    borrow::Cow,
    collections::BTreeMap,
    fmt,
};
pub use self::swiftbar::SwiftBar;

pub mod swiftbar;

#[derive(Debug, Clone, Copy)]
/// A BitBar implementation.
pub enum Flavor {
    /// The original, now discontinued implementation, with just the base features. This is also returned if a plugin is run on its own.
    BitBar,
    /// [SwiftBar](https://swiftbar.app/)
    SwiftBar(SwiftBar),
    //TODO xbar support, blocked on https://github.com/matryer/xbar/issues/753
    //TODO Argos (https://github.com/p-e-w/argos) support? (envar ARGOS_VERSION)
    //TODO kargos (https://github.com/lipido/kargos) support? (needs envar)
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

/// Flavor-specific [`ContentItem`](crate::ContentItem) attributes.
#[derive(Debug)]
#[allow(missing_docs)]
pub enum Attrs {
    SwiftBar(swiftbar::Attrs),
}

impl Attrs {
    pub(crate) fn render<'a>(&'a self, rendered_params: &mut BTreeMap<Cow<'a, str>, Cow<'a, str>>) {
        match self {
            Attrs::SwiftBar(params) => params.render(rendered_params),
        }
    }
}
