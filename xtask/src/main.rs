//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

use std::{env, path::Path};

use serde::Deserialize;
use toml::from_str as from_toml_str;
use xshell::read_file;

mod cargo;
mod ci;
mod flags;
mod release;
mod util;

use self::{ci::CiTask, release::ReleaseTask};

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
        flags::XtaskCmd::Release(cmd) => {
            let mut task = ReleaseTask::new(cmd.name, cmd.version)?;
            task.run()
        }
        flags::XtaskCmd::Publish(cmd) => {
            let mut task = ReleaseTask::new(cmd.name, cmd.version)?;
            task.run()
        }
        flags::XtaskCmd::Ci(ci) => {
            let task = CiTask::new(ci.version)?;
            task.run()
        }
    }
}

#[derive(Debug, Deserialize)]
struct Config {
    /// Credentials to authenticate to GitHub.
    github: GithubConfig,
}

impl Config {
    /// Load a new `Config` from `config.toml`.
    fn load() -> Result<Self> {
        let path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("config.toml");
        let config = read_file(path)?;
        Ok(from_toml_str(&config)?)
    }
}

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
