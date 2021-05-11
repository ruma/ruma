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
        let _p = pushd(&self.project_root)?;

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
        let _p = pushd("crates/ruma")?;
        cmd!("rustup run {MSRV} cargo build --features full").run()
    }

    fn build_stable(&self) -> xshell::Result<()> {
        // 1. Make sure everything compiles
        cmd!("rustup run stable cargo check --workspace --all-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-client --no-default-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-identifiers --no-default-features").run()?;

        // 2. Run tests
        cmd!("rustup run stable cargo test --workspace").run()
    }

    fn build_nightly(&self) -> xshell::Result<()> {
        let fmt_res = cmd!("rustup run nightly cargo fmt -- --check").run();
        let clippy_res = cmd!("rustup run nightly cargo ruma-clippy -D warnings").run();

        let already_installed = cmd!("rustup run nightly cargo sort -V")
            .read()
            .map_or(false, |stdout| !stdout.contains("error"));
        if !already_installed {
            cmd!("rustup run nightly cargo install cargo-sort").run()?;
        }
        let sort_res = cmd!("cargo sort --workspace --grouped --check").run();

        fmt_res.and(clippy_res).and(sort_res)
    }
}
