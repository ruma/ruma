use std::path::PathBuf;

use xshell::pushd;

use crate::{cmd, Metadata, Result};

const MSRV: &str = "1.45";

/// Task to run CI tests.
pub struct CiTask {
    /// Which version of Rust to test against.
    version: Option<String>,

    /// The root of the workspace.
    project_root: PathBuf,
}

impl CiTask {
    pub(crate) fn new(version: Option<String>) -> Result<Self> {
        let project_root = Metadata::load()?.workspace_root;
        Ok(Self { version, project_root })
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
        vec![
            cmd!("rustup run stable cargo test --workspace --quiet").run(),
            cmd!("rustup run stable cargo test -p ruma-identifiers --no-default-features --quiet")
                .run(),
            cmd!("rustup run stable cargo test -p ruma-identifiers --all-features --quiet").run(),
            cmd!("rustup run stable cargo test -p ruma-client-api --all-features --quiet").run(),
            cmd!("rustup run stable cargo check -p ruma-client --no-default-features --quiet")
                .run(),
            cmd!("rustup run stable cargo check -p ruma-client --all-features --quiet").run(),
        ]
        .into_iter()
        .collect()
    }

    fn build_nightly(&self) -> xshell::Result<()> {
        vec![
            cmd!("rustup run nightly cargo fmt -- --check").run(),
            cmd!(
                "rustup run nightly cargo clippy -p ruma
                    --all-targets --all-features --quiet -- -D warnings"
            )
            .run(),
            cmd!(
                "rustup run nightly cargo clippy -p ruma-client
                    --all-targets --quiet -- -D warnings"
            )
            .run(),
        ]
        .into_iter()
        .collect()
    }
}
