use clap::{Args, Subcommand, ValueEnum};
use xshell::Shell;

use crate::{Metadata, Result, cargo::FeatureFilter, cmd};

#[derive(Args)]
pub struct SemverArgs {
    #[clap(subcommand)]
    pub cmd: SemverCmd,
    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,
}

#[derive(Subcommand)]
pub enum SemverCmd {
    /// Check whether there are breaking changes
    BreakingChanges {
        /// The crate to check
        package: String,
        /// The git revision to check against
        baseline_rev: String,
    },
    /// Check whether all cargo features are additive
    AdditiveFeatures {
        /// The crate to check
        package: String,
        /// The features to check
        #[arg(value_enum)]
        features: AdditiveFeaturesKind,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum AdditiveFeaturesKind {
    /// Check whether the stable features are additive against the default features of the crate.
    Stable,
    /// Check whether unstable features are additive against the stable features of the crate.
    Unstable,
}

/// Task to run semver checks.
pub struct SemverTask {
    /// Which command to run.
    cmd: SemverCmd,

    /// Whether to disable colored output.
    no_color: bool,

    /// The shell API to use to run commands.
    sh: Shell,
}

impl SemverTask {
    /// The arguments to disable colored output.
    const NO_COLOR_ARGS: &[&str] = &["color", "never"];

    pub(crate) fn new(cmd: SemverCmd, no_color: bool) -> Result<Self> {
        let sh = Shell::new()?;
        Ok(Self { cmd, no_color, sh })
    }

    pub(crate) fn run(self) -> Result<()> {
        if cmd!(&self.sh, "cargo semver-checks --version").run().is_err() {
            return Err(
                "Could not find cargo-semver-checks. Install it by running `cargo install cargo-semver-checks`".into()
            );
        }

        match &self.cmd {
            SemverCmd::BreakingChanges { package, baseline_rev } => {
                self.breaking_changes(package, baseline_rev)
            }
            SemverCmd::AdditiveFeatures { package, features } => {
                self.additive_features(package, *features)
            }
        }
    }

    /// The arguments to add to disable colored output, if any.
    fn no_color_args(&self) -> &[&str] {
        if self.no_color { Self::NO_COLOR_ARGS } else { &[] }
    }

    /// Check for breaking changes in the given crate, against the given revision.
    fn breaking_changes(&self, package: &str, baseline_rev: &str) -> Result<()> {
        let no_color_args = self.no_color_args();

        cmd!(
            &self.sh,
            "
            rustup run stable cargo semver-checks --release-type minor
                -p {package} --baseline-rev {baseline_rev} {no_color_args...}
            "
        )
        .run()?;

        Ok(())
    }

    /// Check whether cargo features of the given crate are additive.
    fn additive_features(&self, package: &str, features: AdditiveFeaturesKind) -> Result<()> {
        let project_metadata = Metadata::load(&self.sh)?;

        let Some(package_data) = project_metadata.find_package(package) else {
            return Err(format!("Could not find {package} package in project metadata").into());
        };

        if package_data.features.is_empty() {
            // There is nothing to check, return early.
            println!("\nIgnoring additive features check, the {package} crate has no features.\n");
            return Ok(());
        }

        let (baseline_features, current_features) = match features {
            AdditiveFeaturesKind::Stable => {
                let stable_features = package_data.filtered_features(FeatureFilter::Stable);

                if stable_features.is_empty() {
                    // There is nothing to check, return early.
                    println!(
                        "\nIgnoring additive stable features check, \
                         the {package} crate has no stable features.\n"
                    );
                    return Ok(());
                }

                // We need to check against the default features, because ruma-common panics if the
                // default features are not enabled.
                let default_features = package_data.filtered_features(FeatureFilter::Default);

                if default_features == stable_features {
                    // There is nothing to check, return early.
                    println!(
                        "\nIgnoring additive stable features check, the stable features \
                         of the {package} crate are the same as the default features.\n"
                    );
                    return Ok(());
                }

                (default_features.join(","), stable_features.join(","))
            }
            AdditiveFeaturesKind::Unstable => {
                let all_features = package_data.filtered_features(FeatureFilter::All);

                if all_features.is_empty() {
                    // There is nothing to check, return early.
                    println!(
                        "\nIgnoring additive unstable features check, the {package} crate has no public features.\n"
                    );
                    return Ok(());
                }

                let stable_features = package_data.filtered_features(FeatureFilter::Stable);

                if all_features == stable_features {
                    // There is nothing to check, return early.
                    println!(
                        "\nIgnoring additive unstable features check, the {package} crate \
                         only has stable features.\n"
                    );
                    return Ok(());
                }

                (stable_features.join(","), all_features.join(","))
            }
        };

        let mut features_args = vec!["--current-features", &current_features];

        if !baseline_features.is_empty() {
            features_args.push("--baseline-features");
            features_args.push(&baseline_features);
        }

        cmd!(
            &self.sh,
            "
            rustup run stable cargo semver-checks --color never --release-type minor
                -p {package} --only-explicit-features --baseline-rev HEAD
                {features_args...}
            "
        )
        .run()
        .map_err(Into::into)
    }
}
