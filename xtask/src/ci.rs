use std::path::PathBuf;

use clap::{Args, Subcommand};
use xshell::pushd;

use crate::{cmd, Metadata, Result};

mod spec_links;

use spec_links::check_spec_links;

const MSRV: &str = "1.55";

#[derive(Args)]
pub struct CiArgs {
    #[clap(subcommand)]
    pub cmd: Option<CiCmd>,
}

#[derive(Subcommand)]
pub enum CiCmd {
    /// Check crates compile with the MSRV
    Msrv,
    /// Run all the tasks that use the stable version
    Stable,
    /// Check crates compile (stable)
    Check,
    /// Run tests (stable)
    Test,
    /// Run all the tasks that use the nightly version
    Nightly,
    /// Check formatting (nightly)
    Fmt,
    /// Check ruma crate with `full` feature (nightly)
    CheckFull,
    /// Lint code with clippy (nightly)
    Clippy,
    /// Check sorting of dependencies (nightly)
    Dependencies,
    /// Check spec links point to a recent version (nightly)
    SpecLinks,
}

/// Task to run CI tests.
pub struct CiTask {
    /// Which command to run.
    cmd: Option<CiCmd>,

    /// The root of the workspace.
    project_root: PathBuf,
}

impl CiTask {
    pub(crate) fn new(cmd: Option<CiCmd>) -> Result<Self> {
        let project_root = Metadata::load()?.workspace_root;
        Ok(Self { cmd, project_root })
    }

    pub(crate) fn run(self) -> Result<()> {
        let _p = pushd(&self.project_root)?;

        match self.cmd {
            Some(CiCmd::Msrv) => self.build_msrv()?,
            Some(CiCmd::Stable) => self.build_stable()?,
            Some(CiCmd::Check) => self.check()?,
            Some(CiCmd::Test) => self.test()?,
            Some(CiCmd::Nightly) => self.build_nightly()?,
            Some(CiCmd::Fmt) => self.fmt()?,
            Some(CiCmd::CheckFull) => self.check_full()?,
            Some(CiCmd::Clippy) => self.clippy()?,
            Some(CiCmd::Dependencies) => self.dependencies()?,
            Some(CiCmd::SpecLinks) => check_spec_links(&self.project_root.join("crates"))?,
            None => {
                self.build_msrv().and(self.build_stable()).and(self.build_nightly())?;
            }
        }

        Ok(())
    }

    /// Check that the crates compile with the MSRV.
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

    /// Run all the tasks that use the stable version.
    fn build_stable(&self) -> Result<()> {
        self.check()?;
        self.test()
    }

    /// Check that the crates compile with the stable version.
    fn check(&self) -> Result<()> {
        cmd!("rustup run stable cargo check --workspace --all-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-client --no-default-features").run()?;
        cmd!("rustup run stable cargo check -p ruma-common --no-default-features --features client,server").run().map_err(Into::into)
    }

    /// Run tests with the stable version.
    fn test(&self) -> Result<()> {
        let workspace_res = cmd!("rustup run stable cargo test --features __ci").run();
        let events_compat_res =
            cmd!("rustup run stable cargo test -p ruma-common --features events --features compat compat").run();

        workspace_res.and(events_compat_res).map_err(Into::into)
    }

    /// Run all the tasks that use the nightly version.
    fn build_nightly(&self) -> Result<()> {
        // Check formatting
        let fmt_res = self.fmt();
        // Check `ruma` crate with `full` feature (sometimes things only compile with an unstable
        // flag)
        let check_full_res = self.check_full();
        // Lint code with clippy
        let clippy_res = self.clippy();
        // Check dependencies being sorted
        let dependencies_res = self.dependencies();
        // Check that all links point to the same version of the spec
        let spec_links = check_spec_links(&self.project_root.join("crates"));

        fmt_res
            .and(check_full_res)
            .and(clippy_res)
            .and(dependencies_res)
            .map_err(Into::into)
            .and(spec_links)
    }

    /// Check the formatting with the nightly version.
    fn fmt(&self) -> Result<()> {
        cmd!("rustup run nightly cargo fmt -- --check").run().map_err(Into::into)
    }

    /// Check ruma crate with full feature with the nightly version.
    fn check_full(&self) -> Result<()> {
        cmd!("rustup run nightly cargo check -p ruma --features full").run().map_err(Into::into)
    }

    /// Lint the code with clippy with the nightly version.
    fn clippy(&self) -> Result<()> {
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
        clippy_default_res.and(clippy_all_res).map_err(Into::into)
    }

    /// Check the sorting of dependencies with the nightly version.
    fn dependencies(&self) -> Result<()> {
        cmd!(
            "
            rustup run nightly cargo sort
                --workspace --grouped --check
                --order package,lib,features,dependencies,dev-dependencies,build-dependencies
            "
        )
        .run()
        .map_err(Into::into)
    }
}
