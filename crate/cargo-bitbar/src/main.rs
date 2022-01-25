#![deny(rust_2018_idioms, unused, unused_crate_dependencies, unused_import_braces, unused_lifetimes, unused_qualifications, warnings)]
#![forbid(unsafe_code)]

use {
    std::{
        collections::HashMap,
        io::prelude::*,
        path::PathBuf,
    },
    anyhow::Result,
    cargo_metadata::{
        MetadataCommand,
        Package,
    },
    itertools::Itertools as _,
    serde::Deserialize,
    structopt::StructOpt,
};

#[derive(Deserialize)]
struct CustomMetadata {
    #[serde(default)]
    bitbar: BitBarMetadata,
}

#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
enum PluginKind {
    Default,
    Streamable,
}

impl Default for PluginKind {
    fn default() -> Self {
        Self::Default
    }
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct BitBarMetadata {
    #[serde(default, with = "serde_with::rust::double_option")]
    title: Option<Option<String>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    version: Option<Option<String>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    author: Option<Option<String>>,
    author_github: Option<String>,
    #[serde(default, with = "serde_with::rust::double_option")]
    desc: Option<Option<String>>,
    image: Option<String>,
    #[serde(default, with = "serde_with::rust::double_option")]
    dependencies: Option<Option<String>>,
    #[serde(default, with = "serde_with::rust::double_option")]
    abouturl: Option<Option<String>>,
    //TODO xbar variables? (unsure if xbar supports binary plugin metadata)
    //TODO SwiftBar droptypes
    #[serde(default)]
    hide_about: bool,
    #[serde(default)]
    hide_run_in_terminal: bool,
    #[serde(default)]
    hide_last_updated: bool,
    #[serde(default)]
    hide_disable_plugin: bool,
    #[serde(default)]
    hide_swiftbar: bool,
    schedule: Option<String>,
    #[serde(default)]
    refresh_on_open: bool,
    #[serde(default)]
    run_in_bash: bool,
    #[serde(default, rename = "type")]
    kind: PluginKind,
    #[serde(default)]
    environment: HashMap<String, String>,
}

impl BitBarMetadata {
    fn format(self, package: Option<&Package>) -> Result<Vec<u8>> {
        let Self { title, version, author, author_github, desc, image, dependencies, abouturl, hide_about, hide_run_in_terminal, hide_last_updated, hide_disable_plugin, hide_swiftbar, schedule, refresh_on_open, run_in_bash, kind, environment } = self;
        let mut buf = base64::write::EncoderWriter::new(Vec::default(), base64::Config::new(base64::CharacterSet::Standard, true));

        macro_rules! double_option {
            ($field:ident, $fallback:expr) => {
                match $field {
                    Some(Some(field)) => { writeln!(&mut buf, concat!("# <bitbar.", stringify!($field), ">{}</bitbar.", stringify!($field), ">"), field)?; }
                    Some(None) => {}
                    None => { writeln!(&mut buf, concat!("# <bitbar.", stringify!($field), ">{}</bitbar.", stringify!($field), ">"), $fallback)?; }
                }
            };
        }

        macro_rules! triple_option {
            ($field:ident, $fallback:expr) => {
                match $field {
                    Some(Some(field)) => { writeln!(&mut buf, concat!("# <bitbar.", stringify!($field), ">{}</bitbar.", stringify!($field), ">"), field)?; }
                    Some(None) => {}
                    None => if let Some(ref fallback) = $fallback {
                        writeln!(&mut buf, concat!("# <bitbar.", stringify!($field), ">{}</bitbar.", stringify!($field), ">"), fallback)?;
                    },
                }
            };
        }

        triple_option!(title, package.map(|package| &package.name));
        triple_option!(version, package.map(|package| format!("v{}", package.version)));
        triple_option!(author, package.map(|package| package.authors.iter().map(|author| author.rsplit_once(" <").map(|(name, _)| name).unwrap_or(author)).join(", ")));
        if let Some(author_github) = author_github { writeln!(&mut buf, "# <bitbar.author.github>{}</bitbar.author.github>", author_github)?; }
        triple_option!(desc, package.and_then(|package| package.description.as_ref()));
        if let Some(image) = image { writeln!(&mut buf, "# <bitbar.image>{}</bitbar.image>", image)?; }
        double_option!(dependencies, "rust");
        triple_option!(abouturl, package.and_then(|package| package.homepage.as_ref()));
        if hide_about { writeln!(&mut buf, "# <swiftbar.hideAbout>true</swiftbar.hideAbout>")?; }
        if hide_run_in_terminal { writeln!(&mut buf, "# <swiftbar.hideRunInTerminal>true</swiftbar.hideRunInTerminal>")?; }
        if hide_last_updated { writeln!(&mut buf, "# <swiftbar.hideLastUpdated>true</swiftbar.hideLastUpdated>")?; }
        if hide_disable_plugin { writeln!(&mut buf, "# <swiftbar.hideDisablePlugin>true</swiftbar.hideDisablePlugin>")?; }
        if hide_swiftbar { writeln!(&mut buf, "# <swiftbar.hideSwiftBar>true</swiftbar.hideSwiftBar>")?; }
        if let Some(schedule) = schedule { writeln!(&mut buf, "# <swiftbar.schedule>{}</swiftbar.schedule>", schedule)?; }
        if refresh_on_open { writeln!(&mut buf, "# <swiftbar.refreshOnOpen>true</swiftbar.refreshOnOpen>")?; }
        if !run_in_bash { writeln!(&mut buf, "# <swiftbar.runInBash>false</swiftbar.runInBash>")?; }
        match kind {
            PluginKind::Default => {}
            PluginKind::Streamable => { writeln!(&mut buf, "# <swiftbar.type>streamable</swiftbar.type>")?; }
        }
        if !environment.is_empty() {
            writeln!(&mut buf, "# <swiftbar.environment>[{}]</swiftbar.environment", environment.into_iter().map(|(var, default_value)| format!("{}:{}", var, default_value)).join(", "))?;
        }
        Ok(buf.finish()?)
    }
}

#[derive(StructOpt)]
enum Args {
    Bitbar(ArgsInner),
}

#[derive(StructOpt)]
enum ArgsInner {
    /// Read plugin metadata from Cargo.toml and encode it into the given binary.
    Meta {
        /// The path to the Cargo manifest for the package.
        #[structopt(long, parse(from_os_str))]
        manifest: Option<PathBuf>,
        /// The path to the binary that should be edited.
        #[structopt(parse(from_os_str))]
        exe_path: PathBuf,
    },
}

#[paw::main]
fn main(Args::Bitbar(args): Args) -> Result<()> {
    match args {
        ArgsInner::Meta { manifest, exe_path } => {
            let mut metadata_cmd = MetadataCommand::new();
            metadata_cmd.no_deps();
            if let Some(manifest) = manifest {
                metadata_cmd.manifest_path(manifest);
            }
            let metadata = metadata_cmd.exec()?;
            let package = metadata.root_package();
            let custom_metadata = if let Some(package) = package {
                package.metadata.clone()
            } else {
                metadata.workspace_metadata.clone()
            };
            let bitbar_metadata = serde_json::from_value::<CustomMetadata>(custom_metadata)?.bitbar.format(package)?;
            xattr::set(exe_path, "com.ameba.SwiftBar", &bitbar_metadata)?;
        }
    }
    Ok(())
}
