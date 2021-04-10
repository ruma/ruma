#![allow(dead_code)]
use std::{collections::HashMap, path::PathBuf};

use xshell::{pushd, cmd};

use super::{config, CiInfo, CrateCommands, Result};

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
        let CiInfo { versions: valid_versions, tests, default } = config;

        let rust_versions = if let Some(version) = rust_version {
            if valid_versions.contains(&version) {
                vec![version]
            } else {
                return Err("Invalid rust version provided".into());
            }
        } else {
            valid_versions
        };

        let crates: Option<Vec<String>> =
            crates.map(|s| s.split(' ').map(|s| s.to_string()).collect());

        Ok(Self {
            tests: match crates {
                Some(crates) => tests.into_iter().filter(|(k, _)| crates.contains(k)).collect(),
                None => tests.into_iter().filter(|(k, _)| default.contains(k)).collect(),
            },
            project_root,
            rust_versions,
        })
    }
    pub(crate) fn run(self) -> Result<()> {
        for version in self.rust_versions {
            cmd!("rustup default {version}").run()?;
            for (dir, CrateCommands { commands }) in self.tests.iter() {
                let _p = pushd(dir)?;
                for command in commands {
                    cmd!("cargo build {command}").run()?;
                }
            }
        }

        Ok(())
    }
}
