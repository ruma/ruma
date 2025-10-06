use clap::{Args, Subcommand};
use xshell::Shell;

use crate::{cmd, Result};

#[derive(Args)]
pub struct BenchTask {
    /// The package to benchmark.
    ///
    /// All packages are benchmarked if this is not provided.
    #[clap(subcommand)]
    pub package: Option<BenchPackage>,
}

/// The possible packages to benchmark.
#[derive(Subcommand, PartialEq, Eq, PartialOrd, Ord)]
pub enum BenchPackage {
    /// Benchmark `ruma-events` crate.
    RumaEvents,
    /// Benchmark `ruma-state-res` crate.
    RumaStateRes,
}

impl BenchPackage {
    /// Get all the possible packages as a list of package arguments (`-p <package>`) for a cargo
    /// command.
    pub const ALL_PACKAGES_ARGS: &[&str] = &["-p", "ruma-events", "-p", "ruma-state-res"];

    /// Get this package as a package argument (`-p <package>`) for a cargo command.
    pub fn as_package_arg(&self) -> &'static [&'static str] {
        match self {
            Self::RumaEvents => &["-p", "ruma-events"],
            Self::RumaStateRes => &["-p", "ruma-state-res"],
        }
    }
}

impl BenchTask {
    pub(crate) fn run(self) -> Result<()> {
        let packages = match self.package {
            Some(package) => package.as_package_arg(),
            None => BenchPackage::ALL_PACKAGES_ARGS,
        };

        let sh = Shell::new()?;
        cmd!(sh, "rustup run stable cargo bench {packages...} --features __criterion")
            .run()
            .map_err(Into::into)
    }
}
