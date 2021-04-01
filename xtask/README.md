# Ruma tasks

This crate is a helper bin for repetitive tasks during Ruma development, based on [cargo xtask].

To use it, run `cargo xtask [command]` anywhere in the workspace.

Some commands need configuration variables. Copy `config.toml.sample` as `config.toml` and fill
the appropriate fields.

## Commands

- `release [crate]`: Create a signed tag based on the `crate` name and version and create a release
  on GitHub. **Requires all `github` fields in `config.toml`.**

[cargo xtask] : https://github.com/matklad/cargo-xtask
