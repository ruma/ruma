//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

use std::path::PathBuf;

use serde::Deserialize;
use serde_json::from_str as from_json_str;

#[cfg(feature = "default")]
mod cargo;
mod ci;
mod flags;
#[cfg(feature = "default")]
mod release;
#[cfg(feature = "default")]
mod util;

use ci::CiTask;
#[cfg(feature = "default")]
use release::ReleaseTask;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<()> {
    let flags = flags::Xtask::from_env()?;
    match flags.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
            Ok(())
        }
        flags::XtaskCmd::Ci(ci) => {
            let task = CiTask::new(ci.version)?;
            task.run()
        }
        #[cfg(feature = "default")]
        flags::XtaskCmd::Release(cmd) => {
            let mut task = ReleaseTask::new(cmd.name, cmd.version)?;
            task.run()
        }
        #[cfg(feature = "default")]
        flags::XtaskCmd::Publish(cmd) => {
            let mut task = ReleaseTask::new(cmd.name, cmd.version)?;
            task.run()
        }
        #[cfg(not(feature = "default"))]
        _ => {
            Err("This command is only available when xtask is built with default features.".into())
        }
    }
}

/// The metadata of a cargo workspace.
#[derive(Clone, Debug, Deserialize)]
pub struct Metadata {
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
