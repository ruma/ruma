//! See https://github.com/matklad/cargo-xtask/.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`. Run commands as `cargo xtask [command]`.

use std::{
    env,
    path::{Path, PathBuf},
};

use xshell::read_file;

mod flags;
mod release;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<()> {
    let flags = flags::Xtask::from_env()?;
    match flags.subcommand {
        flags::XtaskCmd::Help(_) => {
            println!("{}", flags::Xtask::HELP);
            Ok(())
        }
        flags::XtaskCmd::Release(cmd) => cmd.run(),
    }
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR")).ancestors().nth(1).unwrap().to_path_buf()
}

fn config() -> Result<String> {
    let path = project_root().join("xtask/config.toml");
    match read_file(path) {
        Ok(c) => Ok(c),
        Err(err) => {
            return Err(err)?;
        }
    }
}
