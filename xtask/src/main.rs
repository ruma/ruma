//! See https://github.com/matklad/cargo-xtask/.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

use std::{
    env,
    path::{Path, PathBuf},
};

use serde::Deserialize;
use serde_json::from_str as from_json_str;
use toml::from_str as from_toml_str;
use xshell::{cmd, read_file};

mod flags;
mod release;

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
        flags::XtaskCmd::Release(cmd) => cmd.run(),
    }
}

#[derive(Debug, Deserialize)]
struct CargoMetadata {
    workspace_root: PathBuf,
}

/// Get the project workspace root.
fn project_root() -> Result<PathBuf> {
    let metadata_json = cmd!("cargo metadata --format-version 1").read()?;
    let metadata: CargoMetadata = from_json_str(&metadata_json)?;
    Ok(metadata.workspace_root)
}

#[derive(Debug, Deserialize)]
struct Config {
    /// Credentials to authenticate to GitHub.
    github: GithubConfig,
}

#[derive(Debug, Deserialize)]
struct GithubConfig {
    /// The username to use for authentication.
    user: String,

    /// The personal access token to use for authentication.
    token: String,
}

impl GithubConfig {
    /// Get the GitHub credentials formatted as `user:token`
    fn credentials(&self) -> String {
        format!("{}:{}", self.user, self.token)
    }
}

/// Load the config from `config.toml`.
fn config() -> Result<Config> {
    let path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("config.toml");
    let config = read_file(path)?;
    Ok(from_toml_str(&config)?)
}
