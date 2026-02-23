// Triggers at the `#[clap(subcommand)]` line, but not easily reproducible outside this crate.
#![allow(unused_qualifications)]

use std::path::Path;

use clap::{Args, Subcommand};
use xshell::Shell;

use crate::{Metadata, NIGHTLY, Result, bench::BenchPackage, cargo::FeatureFilter, cmd};

mod reexport_features;
mod spec_links;
mod unused_features;

use reexport_features::check_reexport_features;
use spec_links::check_spec_links;
use unused_features::check_unused_features;

// Keep in sync with README.md, the root Cargo.toml and .github/workflows/ci.yml
const MSRV: &str = "1.88";

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
    /// Check ruma-identifiers with `ruma_identifiers_storage="ArcStr"`
    MsrvOwnedIdArcstr,
    /// Check ruma-identifiers with `ruma_identifiers_storage="SmallVec"`
    MsrvOwnedIdSmallvec,
    /// Check ruma-identifiers with `ruma_identifiers_storage="CompactString"`
    MsrvOwnedIdCompactstring,
    /// Check ruma-identifiers with `ruma_identifiers_storage="ArcIntern"`
    MsrvOwnedIdArcintern,
    /// Run all the tasks that use the stable version
    Stable,
    /// Check all crates with all features (stable)
    StableAll,
    /// Check ruma-common with only the required features (stable)
    StableCommon,
    /// Check all benchmarks (stable)
    StableBenches,
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
    /// Lint client API and unstable features with clippy (nightly)
    ClippyApiClient,
    /// Lint server API and unstable features with clippy (nightly)
    ClippyApiServer,
    /// Lint client features with clippy on a wasm target (nightly)
    ClippyWasm,
    /// Lint almost all features with clippy (nightly)
    ClippyAll,
    /// Lint all benchmarks with clippy (nightly)
    ClippyBenches,
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
            Some(CiCmd::MsrvOwnedIdBox) => self.msrv_owned_id_cfg("Box")?,
            Some(CiCmd::MsrvOwnedIdArc) => self.msrv_owned_id_cfg("Arc")?,
            Some(CiCmd::MsrvOwnedIdArcstr) => self.msrv_owned_id_cfg("ArcStr")?,
            Some(CiCmd::MsrvOwnedIdSmallvec) => self.msrv_owned_id_cfg("SmallVec")?,
            Some(CiCmd::MsrvOwnedIdCompactstring) => self.msrv_owned_id_cfg("CompactString")?,
            Some(CiCmd::MsrvOwnedIdArcintern) => self.msrv_owned_id_cfg("ArcIntern")?,
            Some(CiCmd::Stable) => self.stable()?,
            Some(CiCmd::StableAll) => self.stable_all()?,
            Some(CiCmd::StableCommon) => self.stable_common()?,
            Some(CiCmd::StableBenches) => self.stable_benches()?,
            Some(CiCmd::TestAll) => self.test_all()?,
            Some(CiCmd::TestCompat) => self.test_compat()?,
            Some(CiCmd::TestDoc) => self.test_doc()?,
            Some(CiCmd::Nightly) => self.nightly()?,
            Some(CiCmd::Fmt) => self.fmt()?,
            Some(CiCmd::NightlyFull) => self.nightly_full()?,
            Some(CiCmd::NightlyAll) => self.nightly_all()?,
            Some(CiCmd::ClippyDefault) => self.clippy_default()?,
            Some(CiCmd::ClippyApiClient) => self.clippy_with_features(RumaFeatures::ApiClient)?,
            Some(CiCmd::ClippyApiServer) => self.clippy_with_features(RumaFeatures::ApiServer)?,
            Some(CiCmd::ClippyWasm) => self.clippy_wasm()?,
            Some(CiCmd::ClippyAll) => self.clippy_with_features(RumaFeatures::Compat)?,
            Some(CiCmd::ClippyBenches) => self.clippy_benches()?,
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
        self.stable_common()?;
        self.stable_benches()?;
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

    /// Check all the benchmarks with the stable version.
    fn stable_benches(&self) -> Result<()> {
        let packages = BenchPackage::ALL_PACKAGES_ARGS;

        cmd!(
            &self.sh,
            "rustup run stable cargo check {packages...} --benches --features __criterion"
        )
        .run()
        .map_err(Into::into)
    }

    /// Run tests on all crates with almost all features with the stable version.
    fn test_all(&self) -> Result<()> {
        let features = self.project_metadata.ruma_features(RumaFeatures::All)?;

        cmd!(&self.sh, "rustup run stable cargo test --tests --features {features}")
            .run()
            .map_err(Into::into)
    }

    /// Run tests on all crates with almost all features and the compat features with the stable
    /// version.
    fn test_compat(&self) -> Result<()> {
        let features = self.project_metadata.ruma_features(RumaFeatures::Compat)?;

        cmd!(&self.sh, "rustup run stable cargo test --tests --features {features}")
            .run()
            .map_err(Into::into)
    }

    /// Run doctests on all crates with almost all features with the stable version.
    fn test_doc(&self) -> Result<()> {
        let features = self.project_metadata.ruma_features(RumaFeatures::All)?;

        cmd!(&self.sh, "rustup run stable cargo test --doc --features {features}")
            .run()
            .map_err(Into::into)
    }

    /// Run all the tasks that use the nightly version.
    fn nightly(&self) -> Result<()> {
        self.fmt()?;
        self.nightly_full()?;
        self.clippy_default()?;
        self.clippy_with_features(RumaFeatures::ApiClient)?;
        self.clippy_with_features(RumaFeatures::ApiServer)?;
        self.clippy_wasm()?;
        self.clippy_with_features(RumaFeatures::Compat)?;
        self.clippy_benches()
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

    /// Check ruma-common with `ruma_identifiers_storage="value"`
    fn msrv_owned_id_cfg(&self, value: &str) -> Result<()> {
        cmd!(&self.sh, "rustup run {MSRV} cargo check -p ruma-common")
            .env("RUSTFLAGS", format!("--cfg=ruma_identifiers_storage={value:?}"))
            .run()
            .map_err(Into::into)
    }

    /// Lint default features with clippy with the nightly version.
    fn clippy_default(&self) -> Result<()> {
        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy
                --workspace --all-targets --features full -- -D warnings
            "
        )
        .run()
        .map_err(Into::into)
    }

    /// Lint ruma with clippy with the nightly version and wasm target.
    fn clippy_wasm(&self) -> Result<()> {
        let features = self.project_metadata.ruma_features(RumaFeatures::Wasm)?;

        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy --target wasm32-unknown-unknown
                -p ruma --features {features} -- -D warnings
            "
        )
        .env("CLIPPY_CONF_DIR", ".wasm")
        .run()
        .map_err(Into::into)
    }

    /// Lint almost all features with clippy with the nightly version.
    fn clippy_with_features(&self, features: RumaFeatures) -> Result<()> {
        let features = self.project_metadata.ruma_features(features)?;

        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy
                --workspace --all-targets --features {features} -- -D warnings
            "
        )
        .run()
        .map_err(Into::into)
    }

    /// Lint all benchmarks with clippy with the nightly version.
    fn clippy_benches(&self) -> Result<()> {
        let packages = BenchPackage::ALL_PACKAGES_ARGS;

        cmd!(
            &self.sh,
            "
            rustup run {NIGHTLY} cargo clippy {packages...}
                --benches --features __criterion -- -D warnings
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

/// The features of the ruma package to enable.
#[derive(Debug, Clone, Copy)]
enum RumaFeatures {
    /// Almost all features.
    ///
    /// This includes the `full` feature and the unstable features.
    All,

    /// `All` features and compat features.
    Compat,

    /// The client API and unstable features.
    ApiClient,

    /// The server API and unstable features.
    ApiServer,

    /// Features that we want to test for WASM.
    ///
    /// Includes all the stable features that can be enabled by Matrix clients and all the unstable
    /// features.
    Wasm,
}

impl Metadata {
    /// Get the ruma features from this project metadata as a string.
    ///
    /// Returns a list of comma-separated features.
    ///
    /// Errors if the ruma package cannot be found in the project metadata.
    fn ruma_features(&self, ruma_features: RumaFeatures) -> Result<String> {
        let Some(ruma_package) = self.find_package("ruma") else {
            return Err("Could not find ruma package in project metadata".into());
        };

        let features = match ruma_features {
            RumaFeatures::All => {
                let mut features = ruma_package.filtered_features(FeatureFilter::Unstable);
                features.push("full");
                features
            }
            RumaFeatures::Compat => {
                let mut features = ruma_package.filtered_features(FeatureFilter::UnstableAndCompat);
                features.push("full");
                features
            }
            RumaFeatures::ApiClient => ruma_package.filtered_features(FeatureFilter::ApiClient),
            RumaFeatures::ApiServer => ruma_package.filtered_features(FeatureFilter::ApiServer),
            RumaFeatures::Wasm => {
                let mut features = ruma_package.filtered_features(FeatureFilter::Unstable);
                features.extend([
                    "api",
                    "client-api",
                    "events",
                    "html-matrix",
                    "identity-service-api",
                    "js",
                    "markdown",
                    "rand",
                    "signatures",
                ]);
                features
            }
        };

        Ok(features.join(","))
    }
}
