use std::path::PathBuf;

use xshell::pushd;

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
        {
            let _p = pushd(self.project_root.join("ruma"))?;
            cmd!("rustup run {MSRV} cargo build --features ruma-events,ruma-api,ruma-appservice-api,ruma-client-api,ruma-federation-api,ruma-identity-service-api,ruma-push-gateway-api --quiet").run()?;
        }
        {
            let _p = pushd(self.project_root.join("ruma-client"))?;
            cmd!("rustup run {MSRV} cargo build --quiet").run()?;
        }
        {
            let _p = pushd(self.project_root.join("ruma-identifiers"))?;
            cmd!("rustup run {MSRV} cargo build --no-default-features --quiet").run()?;
        }
        {
            let _p = pushd(self.project_root.join("ruma-identifiers"))?;
            cmd!("rustup run {MSRV} cargo build --all-features --quiet").run()?;
        }
        {
            let _p = pushd(self.project_root.join("ruma-client-api"))?;
            cmd!("rustup run {MSRV} cargo build --all-features --quiet").run()?;
        }
        Ok(())
    }

    fn build_stable(&self) -> Result<()> {
        cmd!("cargo test --all --quiet").run()?;
        {
            let _p = pushd(self.project_root.join("ruma-identifiers"))?;
            cmd!("cargo test --no-default-features --quiet").run()?;
            cmd!("cargo test --all-features --quiet").run()?;
        }
        {
            let _p = pushd(self.project_root.join("ruma-client-api"))?;
            cmd!("cargo check --no-default-features --features http1,http2 --quiet").run()?;
            cmd!("cargo check --no-default-features --features http1,http2,tls-rustls-native-roots --quiet").run()?;
            cmd!("cargo check --no-default-features --features http1,http2,tls-rustls-webpki-roots --quiet").run()?;
        }
        Ok(())
    }

    fn build_nightly(&self) -> Result<()> {
        cmd!("cargo fmt --all --check").run()?;
        {
            let _p = pushd("ruma")?;
            cmd!("cargo clippy --all-targets --all-features --quiet -- D warnings").run()?;
        }
        {
            let _p = pushd("ruma-client")?;
            cmd!("cargo clippy --all-targets --quiet -- D warnings").run()?;
        }
        Ok(())
    }
}
