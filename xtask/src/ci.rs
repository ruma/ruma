use std::path::PathBuf;

use crate::{cmd, Result};

const MSRV: &str = "1.45";

macro_rules! run_in {
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
        let _p = xshell::pushd(&self.project_root)?;

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
        run_in!("ruma", "rustup run {MSRV} cargo build --features full --quiet");

        Ok(())
    }

    fn build_stable(&self) -> Result<()> {
        cmd!("rustup run stable cargo test --workspace --quiet").run()?;

        {
            let _p = xshell::pushd("ruma-identifiers")?;
            cmd!("rustup run stable cargo test --no-default-features --quiet").run()?;
            cmd!("rustup run stable cargo test --all-features --quiet").run()?;
        }

        run_in!("ruma-client-api", "rustup run stable cargo test --all-features --quiet");

        {
            let _p = xshell::pushd("ruma-client")?;
            cmd!("rustup run stable cargo check --no-default-features --features http1,http2 --quiet")
                .run()?;
            cmd!("rustup run stable cargo check --no-default-features --features http1,http2,tls-rustls-native-roots --quiet")
                .run()?;
            cmd!("rustup run stable cargo check --no-default-features --features http1,http2,tls-rustls-webpki-roots --quiet")
                .run()?;
        }

        Ok(())
    }

    fn build_nightly(&self) -> Result<()> {
        cmd!("rustup run nightly cargo fmt --all").run()?;
        run_in!(
            "ruma",
            "rustup run nightly cargo clippy --all-targets --all-features --quiet -- -D warnings"
        );
        run_in!(
            "ruma-client",
            "rustup run nightly cargo clippy --all-targets --quiet -- -D warnings"
        );

        Ok(())
    }
}
