#![allow(clippy::vec_init_then_push)]

use std::path::PathBuf;

use xshell::pushd;

use crate::{cmd, Result};

const MSRV: &str = "1.45";

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
        let _p = xshell::pushd(&self.project_root)?;

        match self.version.as_deref() {
            Some("msrv") => self.build_msrv()?,
            Some("stable") => self.build_stable()?,
            Some("nightly") => self.build_nightly()?,
            Some(_) => return Err("Wrong Rust version specified.".into()),
            None => {
                self.build_msrv().and(self.build_stable()).and(self.build_nightly())?;
            }
        }

        Ok(())
    }

    fn build_msrv(&self) -> xshell::Result<()> {
        let _p = pushd("ruma")?;
        cmd!("rustup run {MSRV} cargo build --features full --quiet").run()
    }

    fn build_stable(&self) -> xshell::Result<()> {
        let mut r = Vec::new();
        r.push(cmd!("rustup run stable cargo test --workspace --quiet").run());

        {
            let _p = pushd("ruma-identifiers")?;
            r.push(cmd!("rustup run stable cargo test --no-default-features --quiet").run());
            r.push(cmd!("rustup run stable cargo test --all-features --quiet").run());
        }

        {
            let _p = pushd("ruma-client-api");
            r.push(cmd!("rustup run stable cargo test --all-features --quiet").run());
        }

        {
            let _p = pushd("ruma-client")?;
            r.push(
                cmd!(
                    "rustup run stable cargo check
                        --no-default-features --features http1,http2 --quiet"
                )
                .run(),
            );
            r.push(
                cmd!(
                    "rustup run stable cargo check
                        --no-default-features --features http1,http2,tls-rustls-native-roots
                        --quiet"
                )
                .run(),
            );
            r.push(
                cmd!(
                    "rustup run stable cargo check
                        --no-default-features --features http1,http2,tls-rustls-webpki-roots
                        --quiet"
                )
                .run(),
            );
        }

        r.into_iter().collect()
    }

    fn build_nightly(&self) -> xshell::Result<()> {
        let mut r = Vec::new();
        r.push(cmd!("rustup run nightly cargo fmt --all").run());

        {
            let _p = pushd("ruma");
            r.push(
                cmd!(
                    "rustup run nightly cargo clippy
                        --all-targets --all-features --quiet -- -D warnings"
                )
                .run(),
            );
        }

        {
            let _p = pushd("ruma-client");
            r.push(
                cmd!("rustup run nightly cargo clippy --all-targets --quiet -- -D warnings").run(),
            );
        }

        r.into_iter().collect()
    }
}
