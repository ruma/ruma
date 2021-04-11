use std::{path::PathBuf, process::Command};

use crate::{cmd, Result};

const MSRV: &'static str = "1.45";

/// Task to run CI tests.
pub struct CiTask {
    /// Which version of Rust to test against.
    version: Option<String>,

    /// The root of the workspace.
    project_root: PathBuf,
}

impl CiTask {
    pub(crate) fn new(version: Option<String>, project_root: PathBuf) -> Self {
        Self { version, project_root }
    }

    pub(crate) fn run(self) -> Result<()> {
        match self.version.as_ref().map(|s| s.as_str()) {
            Some("msrv") => self.build_msrv(),
            Some("stable") => self.build_stable(),
            Some("nightly") => self.build_nightly(),
            Some(_) => Err("Wrong Rust version specified.".into()),
            None => {
                self.build_msrv()?;
                self.build_stable()?;
                self.build_nightly()
            }
        }
    }

    fn build_msrv(&self) -> Result<()> {
        self.run_in_dir(
            "ruma",
            format!("rustup run {} {}", MSRV,
            r#"cargo build \
--features ruma-events,ruma-api,ruma-appservice-api,ruma-client-api,ruma-federation-api,ruma-identity-service-api,ruma-push-gateway-api \
--quiet"#).as_str(),
        )?;
        self.run_in_dir(
            "ruma-client",
            format!("rustup run {} {}", MSRV, "cargo build --quiet").as_str(),
        )?;
        self.run_in_dir(
            "ruma-identifiers",
            format!("rustup run {} {}", MSRV, "cargo build --no-default-features --quiet").as_str(),
        )?;
        self.run_in_dir(
            "ruma-identifiers",
            format!("rustup run {} {}", MSRV, "cargo build --all-features --quiet").as_str(),
        )?;
        self.run_in_dir(
            "ruma-client-api",
            format!("rustup run {} {}", MSRV, "cargo build --all-features --quiet").as_str(),
        )
    }

    fn build_stable(&self) -> Result<()> {
        self.run_in_dir("", "cargo test --all --quiet")?;
        self.run_in_dir("ruma-identifiers", "cargo test --no-default-features --quiet")?;
        self.run_in_dir("ruma-identifiers", "cargo test --all-features --quiet")?;
        self.run_in_dir("ruma-client-api", "cargo check --all-targets --quiet")?;
        self.run_in_dir(
            "ruma-client",
            "cargo check --no-default-features --features http1,http2 --quiet",
        )?;
        self.run_in_dir(
            "ruma-client",
            "cargo check --no-default-features --features http1,http2,tls-rustls-native-roots --quiet",
        )?;
        self.run_in_dir(
            "ruma-client",
            "cargo check --no-default-features --features http1,http2,tls-rustls-webpki-roots --quiet",
        )
    }

    fn build_nightly(&self) -> Result<()> {
        self.run_in_dir("", "cargo fmt --all --check")?;
        self.run_in_dir("ruma", "cargo clippy --all-targets --all-features --quiet -- D warnings")?;
        self.run_in_dir("ruma-client", "cargo clippy --all-targets --quiet -- D warnings")
    }

    fn run_in_dir(&self, crate_name: &str, command: &str) -> Result<()> {
        let _p = xshell::pushd(self.project_root.join(crate_name))?;
        cmd!("{command}").run()?;
        Ok(())
    }
}
