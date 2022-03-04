use std::path::PathBuf;

use xshell::pushd;

use crate::{cmd, Metadata, Result};

mod spec_links;

use spec_links::check_spec_links;

const MSRV: &str = "1.55";

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

    fn build_msrv(&self) -> Result<()> {
        // Check all crates with all features except
        // * ruma (would pull in ruma-signatures)
        // * ruma-client (tested only with client-api feature due to most / all optional HTTP client
        //   deps having less strict MSRV)
        // * ruma-signatures (MSRV exception)
        // * xtask (no real reason to enforce an MSRV for it)
        cmd!(
            "rustup run {MSRV} cargo check --workspace --all-features
                --exclude ruma
                --exclude ruma-client
                --exclude ruma-signatures
                --exclude xtask"
        )
        .run()?;

        // Check ruma-client crate with default features
        cmd!("rustup run {MSRV} cargo check -p ruma-client --features client-api").run()?;

        // Check ruma crate with default features
        cmd!("rustup run {MSRV} cargo check -p ruma").run().map_err(Into::into)
    }

    fn build_stable(&self) -> Result<()> {
        // 1. Make sure everything compiles
        cmd!("rustup run stable cargo check --workspace --all-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-client --no-default-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-common").run()?;

        // 2. Run tests
        let workspace_res = cmd!("rustup run stable cargo test --features __ci").run();
        let events_compat_res =
            cmd!("rustup run stable cargo test -p ruma-common --features events --features compat compat").run();

        workspace_res.and(events_compat_res).map_err(Into::into)
    }

    fn build_nightly(&self) -> Result<()> {
        // Check formatting
        let fmt_res = cmd!("rustup run nightly cargo fmt -- --check").run();
        // Check `ruma` crate with `full` feature (sometimes things only compile with an unstable
        // flag)
        let check_full_res = cmd!("rustup run nightly cargo check -p ruma --features full").run();
        // Check everything with default features with clippy
        let clippy_default_res = cmd!(
            "
            rustup run nightly cargo clippy
                --workspace --all-targets --features=full -- -D warnings
            "
        )
        .run();
        // Check everything with almost all features with clippy
        let clippy_all_res = cmd!(
            "
            rustup run nightly cargo clippy
                --workspace --all-targets --features=__ci,compat -- -D warnings
            "
        )
        .run();
        // Check dependencies being sorted
        let sort_res = cmd!(
            "
            rustup run nightly cargo sort
                --workspace --grouped --check
                --order package,lib,features,dependencies,dev-dependencies,build-dependencies
            "
        )
        .run();
        // Check that all links point to the same version of the spec
        let spec_links = check_spec_links(&self.project_root.join("crates"));

        fmt_res
            .and(check_full_res)
            .and(clippy_default_res)
            .and(clippy_all_res)
            .and(sort_res)
            .map_err(Into::into)
            .and(spec_links)
    }
}
