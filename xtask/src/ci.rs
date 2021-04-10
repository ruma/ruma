#![allow(dead_code)]
use std::{collections::HashMap, path::PathBuf};

use super::{config, CiInfo, CrateCommands, Result};

macro_rules! cmd {
    ($cmd:tt) => {
        super::cmd!($cmd)
    };
    ($dir:expr, $cmd:tt) => {{
        let _p = xshell::pushd($dir);
        super::cmd!($cmd)
    }};
}

/// Task to run CI Tests
pub struct CiTask {
    /// Commands to run for the CI tests. Keys are the crate names and values are structs that
    /// store the command to run.
    tests: HashMap<String, CrateCommands>,

    /// The root of the workspace.
    project_root: PathBuf,

    /// The version of rust. Valid options are:
    /// - "1.45"
    /// - "nightly"
    /// - "stable"
    ///
    /// `None` means run all versions.
    rust_versions: Vec<String>,
}

impl CiTask {
    pub(crate) fn new(
        crates: Option<String>,
        project_root: PathBuf,
        rust_version: Option<String>,
    ) -> Result<Self> {
        let config = config()?.ci;
        let CiInfo { versions: valid_versions, tests } = config;

        let rust_versions = if let Some(version) = rust_version {
            if valid_versions.contains(&version) {
                return Err("Invalid rust version provided".into());
            } else {
                vec![version]
            }
        } else {
            valid_versions
        };

        let crates: Option<Vec<String>> =
            crates.map(|s| s.split(' ').map(|s| s.to_string()).collect());

        Ok(Self {
            tests: match crates {
                Some(crates) => tests.into_iter().filter(|(k, _)| crates.contains(k)).collect(),
                None => tests,
            },
            project_root,
            rust_versions,
        })
    }
    pub(crate) fn run(self) -> Result<()> {
        for version in self.rust_versions {
            cmd!("rustup default {version}").run()?;
            for (dir, CrateCommands { command }) in self.tests.iter() {
                let command = command.as_str();
                cmd!(dir, "cargo build {command}").run()?;
            }
        }

        Ok(())
    }
}
