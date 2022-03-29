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
            rustup run nightly cargo doc --no-deps --workspace
            --exclude ruma-macros --exclude ruma-identifiers-validation --exclude xtask
            --all-features -Zrustdoc-map
            "
        )
        .env("RUSTDOCFLAGS", rustdocflags);

        if self.open {
            cmd = cmd.arg("--open");
        }

        cmd.run()?;

        Ok(())
    }
}
