// Triggers at the `#[clap(subcommand)]` line, but not easily reproducible outside this crate.
#![allow(unused_qualifications)]

use std::path::Path;

use clap::{Args, Subcommand};
use xshell::Shell;

use crate::{cmd, Metadata, Result, NIGHTLY};

mod reexport_features;
mod spec_links;
mod unused_features;

use reexport_features::check_reexport_features;
use spec_links::check_spec_links;
use unused_features::check_unused_features;

const MSRV: &str = "1.75";

#[derive(Args)]
pub struct CiArgs {
    #[clap(subcommand)]
    pub cmd: Option<CiCmd>,
}

#[derive(Subcommand)]
pub enum CiCmd {
    /// Check crates compile with the MSRV
    Msrv,
    /// Check all crates with all features (msrv)
    MsrvAll,
    /// Check ruma crate with default features (msrv)
    MsrvRuma,
    /// Check ruma-identifiers with `ruma_identifiers_storage="Box"`
    MsrvOwnedIdBox,
    /// Check ruma-identifiers with `ruma_identifiers_storage="Arc"`
    MsrvOwnedIdArc,
    /// Run all the tasks that use the stable version
    Stable,
    /// Check all crates with all features (stable)
    StableAll,
    /// Check ruma-client without default features (stable)
    StableClient,
    /// Check ruma-common with only the required features (stable)
    StableCommon,
    /// Run all tests with almost all features (stable)
    TestAll,
    /// Run all tests with almost all features, including the compat features (stable)
    TestCompat,
    /// Run doc tests with almost all features (stable)
    TestDoc,
    /// Run all the tasks that use the nightly version
    Nightly,
    /// Check formatting (nightly)
    Fmt,
    /// Check ruma crate with `full` features (nightly)
    NightlyFull,
    /// Check all crates with all features (nightly)
    NightlyAll,
    /// Lint default features with clippy (nightly)
    ClippyDefault,
    /// Lint client features with clippy on a wasm target (nightly)
    ClippyWasm,
    /// Lint almost all features with clippy (nightly)
    ClippyAll,
    /// Run all lints that don't need compilation
    Lint,
    /// Check sorting of dependencies (lint)
    Dependencies,
    /// Check spec links point to a recent version (lint)
    SpecLinks,
    /// Check all cargo features of sub-crates can be enabled from ruma (lint)
    ReexportFeatures,
    /// Check typos
    Typos,
    /// Check whether there are unused cargo features (lint)
    UnusedFeatures,
}

/// Task to run CI tests.
pub struct CiTask {
    /// Which command to run.
    cmd: Option<CiCmd>,

    /// The metadata of the workspace.
    project_metadata: Metadata,

    /// The shell API to use to run commands.
    sh: Shell,
}

impl CiTask {
    pub(crate) fn new(cmd: Option<CiCmd>) -> Result<Self> {
        let sh = Shell::new()?;
        let project_metadata = Metadata::load(&sh)?;
        Ok(Self { cmd, sh, project_metadata })
    }

    fn project_root(&self) -> &Path {
        &self.project_metadata.workspace_root
    }

    pub(crate) fn run(self) -> Result<()> {
        let _p = self.sh.push_dir(self.project_root());

        match self.cmd {
            Some(CiCmd::Msrv) => self.msrv()?,
            Some(CiCmd::MsrvAll) => self.msrv_all()?,
            Some(CiCmd::MsrvRuma) => self.msrv_ruma()?,
            Some(CiCmd::MsrvOwnedIdBox) => self.msrv_owned_id_box()?,
            Some(CiCmd::MsrvOwnedIdArc) => self.msrv_owned_id_arc()?,
            Some(CiCmd::Stable) => self.stable()?,
            Some(CiCmd::StableAll) => self.stable_all()?,
            Some(CiCmd::StableClient) => self.stable_client()?,
            Some(CiCmd::StableCommon) => self.stable_common()?,
            Some(CiCmd::TestAll) => self.test_all()?,
            Some(CiCmd::TestCompat) => self.test_compat()?,
            Some(CiCmd::TestDoc) => self.test_doc()?,
            Some(CiCmd::Nightly) => self.nightly()?,
            Some(CiCmd::Fmt) => self.fmt()?,
            Some(CiCmd::NightlyFull) => self.nightly_full()?,
            Some(CiCmd::NightlyAll) => self.nightly_all()?,
            Some(CiCmd::ClippyDefault) => self.clippy_default()?,
            Some(CiCmd::ClippyWasm) => self.clippy_wasm()?,
            Some(CiCmd::ClippyAll) => self.clippy_all()?,
            Some(CiCmd::Lint) => self.lint()?,
            Some(CiCmd::Dependencies) => self.dependencies()?,
            Some(CiCmd::SpecLinks) => check_spec_links(&self.project_root().join("crates"))?,
            Some(CiCmd::ReexportFeatures) => check_reexport_features(&self.project_metadata)?,
            Some(CiCmd::Typos) => self.typos()?,
            Some(CiCmd::UnusedFeatures) => check_unused_features(&self.sh, &self.project_metadata)?,
            None => {
                self.msrv()
                    .and(self.stable())
                    .and(self.nightly())
                    .and(self.lint())
                    .and(self.typos())?;
            }
        }

        Ok(())
    }

    /// Check that the crates compile with the MSRV.
    fn msrv(&self) -> Result<()> {
        self.msrv_all()?;
        self.msrv_ruma()
    }

    /// Check all crates with all features with the MSRV, except:
    /// * ruma (would pull in ruma-signatures)
    /// * ruma-macros (it's still pulled as a dependency but don't want to enable its nightly-only
    ///   internal feature here)
    /// * ruma-signatures (MSRV exception)
    /// * xtask (no real reason to enforce an MSRV for it)
    fn msrv_all(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "rustup run {MSRV} cargo check --workspace --all-features
                --exclude ruma
                --exclude ruma-macros
                --exclude ruma-signatures
                --exclude xtask"
        )
        .run()
        .map_err(Into::into)
    }

    /// Check ruma crate with default features with the MSRV.
    fn msrv_ruma(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run {MSRV} cargo check -p ruma").run().map_err(Into::into)
    }

    /// Run all the tasks that use the stable version.
    fn stable(&self) -> Result<()> {
        self.stable_all()?;
        self.stable_client()?;
        self.stable_common()?;
        self.test_all()?;
        self.test_doc()?;
        Ok(())
    }

    /// Check all crates with all features with the stable version.
    fn stable_all(&self) -> Result<()> {
        // ruma-macros is pulled in as a dependency, but excluding it on the command line means its
        // features don't get activated. It has only a single feature, which is nightly-only.
        cmd!(
            &self.sh,
            "rustup run stable cargo check
                --workspace --all-features --exclude ruma-macros"
        )
        .run()
        .map_err(Into::into)
    }

    /// Check ruma-client without default features with the stable version.
    fn stable_client(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run stable cargo check -p ruma-client --no-default-features")
            .run()
            .map_err(Into::into)
    }

    /// Check ruma-common with onjy the required features with the stable version.
    fn stable_common(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "rustup run stable cargo check -p ruma-common
                --no-default-features --features client,server"
        )
        .run()
        .map_err(Into::into)
    }

    /// Run tests on all crates with almost all features with the stable version.
    fn test_all(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run stable cargo test --tests --features __ci")
            .run()
            .map_err(Into::into)
    }

    /// Run tests on all crates with almost all features and the compat features with the stable
    /// version.
    fn test_compat(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run stable cargo test --tests --features __ci,compat")
            .run()
            .map_err(Into::into)
    }

    /// Run doctests on all crates with almost all features with the stable version.
    fn test_doc(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run stable cargo test --doc --features __ci")
            .run()
            .map_err(Into::into)
    }

    /// Run all the tasks that use the nightly version.
    fn nightly(&self) -> Result<()> {
        self.fmt()?;
        self.nightly_full()?;
        self.clippy_default()?;
        self.clippy_wasm()?;
        self.clippy_all()
    }

    /// Check the formatting with the nightly version.
    fn fmt(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run {NIGHTLY} cargo fmt -- --check").run().map_err(Into::into)
    }

    /// Check ruma crate with full feature with the nightly version.
    fn nightly_full(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run {NIGHTLY} cargo check -p ruma --features full")
            .run()
            .map_err(Into::into)
    }

    /// Check all crates with all features with the nightly version.
    ///
    /// Also checks that all features that are used in the code exist.
    fn nightly_all(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo check
                --workspace --all-features -Z unstable-options
            "
        )
        .env(
            "RUSTFLAGS",
            "-Z crate-attr=feature(type_privacy_lints) \
             -D private_bounds,private_interfaces,unnameable_types,warnings",
        )
        .run()
        .map_err(Into::into)
    }

    /// Check ruma-common with `ruma_identifiers_storage="Box"`
    fn msrv_owned_id_box(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run {MSRV} cargo check -p ruma-common")
            .env("RUSTFLAGS", "--cfg=ruma_identifiers_storage=\"Box\"")
            .run()
            .map_err(Into::into)
    }

    /// Check ruma-common with `ruma_identifiers_storage="Arc"`
    fn msrv_owned_id_arc(&self) -> Result<()> {
        cmd!(&self.sh, "rustup run {MSRV} cargo check -p ruma-common")
            .env("RUSTFLAGS", "--cfg=ruma_identifiers_storage=\"Arc\"")
            .run()
            .map_err(Into::into)
    }

    /// Lint default features with clippy with the nightly version.
    fn clippy_default(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy
                --workspace --all-targets --features=full -- -D warnings
            "
        )
        .run()
        .map_err(Into::into)
    }

    /// Lint ruma with clippy with the nightly version and wasm target.
    fn clippy_wasm(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy --target wasm32-unknown-unknown -p ruma --features
                __unstable-mscs,api,canonical-json,client-api,events,html-matrix,identity-service-api,js,markdown,rand,signatures,unstable-unspecified -- -D warnings
            "
        )
        .env("CLIPPY_CONF_DIR", ".wasm")
        .run()
        .map_err(Into::into)
    }

    /// Lint almost all features with clippy with the nightly version.
    fn clippy_all(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy
                --workspace --all-targets --features=__ci,compat -- -D warnings
            "
        )
        .run()
        .map_err(Into::into)
    }

    /// Run all lints that don't need compilation.
    fn lint(&self) -> Result<()> {
        // Check dependencies being sorted
        let dependencies_res = self.dependencies();
        // Check that all links point to the same version of the spec
        let spec_links_res = check_spec_links(&self.project_root().join("crates"));
        // Check that all cargo features of sub-crates can be enabled from ruma.
        let reexport_features_res = check_reexport_features(&self.project_metadata);
        // Check whether there are unused cargo features.
        let unused_features_res = check_unused_features(&self.sh, &self.project_metadata);

        dependencies_res.and(spec_links_res).and(reexport_features_res).and(unused_features_res)
    }

    /// Check the sorting of dependencies with the nightly version.
    fn dependencies(&self) -> Result<()> {
        if cmd!(&self.sh, "cargo sort --version").run().is_err() {
            return Err(
                "Could not find cargo-sort. Install it by running `cargo install cargo-sort`"
                    .into(),
            );
        }
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo sort
                --workspace --grouped --check
                --order package,lib,features,dependencies,target,dev-dependencies,build-dependencies
            "
        )
        .run()
        .map_err(Into::into)
    }

    /// Check the typos.
    fn typos(&self) -> Result<()> {
        if cmd!(&self.sh, "typos --version").run().is_err() {
            return Err(
                "Could not find typos. Install it by running `cargo install typos-cli`".into()
            );
        }
        cmd!(&self.sh, "typos").run().map_err(Into::into)
    }
}
