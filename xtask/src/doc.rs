use clap::Args;

use crate::{cmd, Result};

#[derive(Args)]
pub struct DocTask {
    /// Open the browser when the docs are built.
    #[clap(long)]
    pub open: bool,

    /// Fail on warnings.
    #[clap(long)]
    pub deny_warnings: bool,
}

impl DocTask {
    pub(crate) fn run(self) -> Result<()> {
        let mut rustdocflags = "--enable-index-page -Zunstable-options --cfg docsrs".to_owned();
        if self.deny_warnings {
            rustdocflags += " -Dwarnings";
        }

        // Keep in sync with .github/workflows/docs.yml
        let mut cmd = cmd!(
            "
            rustup run nightly cargo doc --all-features --no-deps --workspace
            --exclude ruma-macros --exclude ruma-identifiers-validation --exclude xtask
            "
        )
        // Work around https://github.com/rust-lang/cargo/issues/10744
        .env("CARGO_TARGET_APPLIES_TO_HOST", "true")
        .env("RUSTDOCFLAGS", rustdocflags);

        if self.open {
            cmd = cmd.arg("--open");
        }

        cmd.run()?;

        Ok(())
    }
}
