//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

use std::{
    env,
    path::{Path, PathBuf},
    collections::HashMap
};

use serde::Deserialize;
use serde_json::from_str as from_json_str;
use toml::from_str as from_toml_str;
use xshell::read_file;

mod ci;
mod flags;
mod release;

use self::release::ReleaseTask;
use self::ci::CiTask;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<()> {
    let project_root = project_root()?;

    let flags = flags::Xtask::from_env()?;
    match flags.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
            Ok(())
        }
        flags::XtaskCmd::Release(cmd) => {
            let task = ReleaseTask::new(cmd.name, project_root)?;
            task.run()
        }
        flags::XtaskCmd::Ci(ci) => {
            println!(
                "CI Tests are running on {} using {}...",
                ci.crates.as_ref().unwrap_or(&"all".to_string()),
                ci.version.as_ref().unwrap_or(&"all".to_string()),
            );

            let task = CiTask::new(ci.crates, project_root, ci.version)?;
            task.run()
        }
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
    github: Option<GithubConfig>,

    /// Keep information about CI.
    ci: CiInfo
}

#[derive(Debug, Deserialize)]
struct GithubConfig {
    /// The username to use for authentication.
    user: String,

    /// The personal access token to use for authentication.
    token: String,
}

#[derive(Debug, Deserialize)]
struct CiInfo {
    /// Versions of rust that are allowed.
    versions: Vec<String>,

    /// Default set of crates to compile when using xtask.
    default: Vec<String>,

    /// Commands to run for the CI tests. Keys are the crate names and values are structs that
    /// store the command to run.
    tests: HashMap<String, CrateCommands>
}

#[derive(Debug, Deserialize)]
struct CrateCommands {
    /// The command to compile.
    commands: Vec<String>,
}

/// Load the config from `config.toml`.
fn config() -> Result<Config> {
    let path = Path::new(&env!("CARGO_MANIFEST_DIR")).join("config.toml");
    println!("{:?}", path);
    let config = read_file(path)?;
    Ok(from_toml_str(&config)?)
}

#[macro_export]
macro_rules! cmd {
    ($cmd:tt) => {
        xshell::cmd!($cmd).echo_cmd(false)
    };
}
