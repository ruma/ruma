//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

#![allow(clippy::exhaustive_structs)]

use std::path::PathBuf;

use clap::{Parser, Subcommand};
use serde::Deserialize;
use serde_json::from_str as from_json_str;

#[cfg(feature = "default")]
mod cargo;
mod ci;
mod doc;
#[cfg(feature = "default")]
mod release;
#[cfg(feature = "default")]
mod util;

use ci::{CiArgs, CiTask};
use doc::DocTask;
#[cfg(feature = "default")]
use release::{ReleaseArgs, ReleaseTask};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

#[derive(Parser)]
struct Xtask {
    #[clap(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run continuous integration checks
    Ci(CiArgs),
    /// Build the docs
    Doc(DocTask),
    /// Publish a new version of a crate on crates.io, `publish` can be used as an alias
    #[cfg(feature = "default")]
    #[clap(alias = "publish")]
    Release(ReleaseArgs),
}

fn main() -> Result<()> {
    match Xtask::parse().cmd {
        Command::Ci(args) => {
            let ci = CiTask::new(args.cmd)?;
            ci.run()
        }
        Command::Doc(doc) => doc.run(),
        #[cfg(feature = "default")]
        Command::Release(args) => {
            let mut task = ReleaseTask::new(args.package, args.version, args.dry_run)?;
            task.run()
        }
    }
}

/// The metadata of a cargo workspace.
#[derive(Clone, Debug, Deserialize)]
struct Metadata {
    pub workspace_root: PathBuf,
    #[cfg(feature = "default")]
    pub packages: Vec<cargo::Package>,
}

impl Metadata {
    /// Load a new `Metadata` from the command line.
    pub fn load() -> Result<Metadata> {
        let metadata_json = cmd!("cargo metadata --no-deps --format-version 1").read()?;
        Ok(from_json_str(&metadata_json)?)
    }
}

#[cfg(feature = "default")]
#[derive(Debug, Deserialize)]
struct Config {
    /// Credentials to authenticate to GitHub.
    github: GithubConfig,
}

#[cfg(feature = "default")]
impl Config {
    /// Load a new `Config` from `config.toml`.
    fn load() -> Result<Self> {
        use std::{env, path::Path};

        let path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("config.toml");
        let config = xshell::read_file(path)?;
        Ok(toml::from_str(&config)?)
    }
}

#[cfg(feature = "default")]
#[derive(Debug, Deserialize)]
struct GithubConfig {
    /// The username to use for authentication.
    user: String,

    /// The personal access token to use for authentication.
    token: String,
}

#[macro_export]
macro_rules! cmd {
    ($cmd:tt) => {
        xshell::cmd!($cmd).echo_cmd(false)
    };
}
