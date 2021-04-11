use std::path::PathBuf;

use crate::{cmd, Result};

const MSRV: &str = "1.45";

macro_rules! cmd_in {
    ($dir:expr, $($c:tt),+ $(,)?) => {{
        let _p = xshell::pushd($dir)?;
        $(super::cmd!($c).run()?;)+
    }};
}

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
        match self.version.as_deref() {
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
        cmd_in!(
            self.project_root.join("ruma"),
            "rustup run {MSRV} cargo build --features ruma-events,ruma-api,ruma-appservice-api,ruma-client-api,ruma-federation-api,ruma-identity-service-api,ruma-push-gateway-api --quiet",
        );
        cmd_in!(self.project_root.join("ruma-client"), "rustup run {MSRV} cargo build --quiet");
        cmd_in!(
            self.project_root.join("ruma-identifiers"),
            "rustup run {MSRV} cargo build --no-default-features --quiet"
        );
        cmd_in!(
            self.project_root.join("ruma-identifiers"),
            "rustup run {MSRV} cargo build --all-features --quiet"
        );
        cmd_in!(
            self.project_root.join("ruma-client-api"),
            "rustup run {MSRV} cargo build --all-features --quiet"
        );
        Ok(())
    }

    fn build_stable(&self) -> Result<()> {
        cmd!("cargo test --all --quiet").run()?;
        {
            let _p = xshell::pushd(self.project_root.join("ruma-identifiers"))?;
            cmd!("cargo test --no-default-features --quiet").run()?;
            cmd!("cargo test --all-features --quiet").run()?;
        }
        {
            let _p = xshell::pushd(self.project_root.join("ruma-client-api"))?;
            cmd!("cargo check --no-default-features --features http1,http2 --quiet").run()?;
            cmd!("cargo check --no-default-features --features http1,http2,tls-rustls-native-roots --quiet").run()?;
            cmd!("cargo check --no-default-features --features http1,http2,tls-rustls-webpki-roots --quiet").run()?;
        }
        Ok(())
    }

    fn build_nightly(&self) -> Result<()> {
        cmd!("cargo fmt --all").run()?;
        cmd_in!("ruma", "cargo clippy --all-targets --all-features --quiet -- -D warnings");
        cmd_in!("ruma-client", "cargo clippy --all-targets --quiet -- -D warnings");
        Ok(())
    }
}
