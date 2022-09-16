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

        let mut cmd = cmd!("rustup run nightly cargo doc --all-features --no-deps")
            .env("RUSTDOCFLAGS", rustdocflags);

        if self.open {
            cmd = cmd.arg("--open");
        }

        cmd.run()?;

        Ok(())
    }
}
