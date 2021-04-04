# How to release new versions of the Ruma crates

## General

* `ruma-identifiers-validation` and the `-macros` crates don't get their own
  release commit or tag since they are internal packages.
* Macro crates are versioned identically to their "parent" crate and need to be
  released before the parent crate.
* `ruma-identifiers-validation` is released before `ruma-identifiers-macros`
  and `ruma-identifiers`.

## Creating a release commit

*To be automated in the future, see https://github.com/ruma/ruma/issues/452.*

Update `Cargo.toml` of the relevant package, and update the dependency on this
package in other crates in the workspace (if applicable):

* When doing a prerelease (e.g. `0.10.0-beta.1`), depend on it with an exact
  version requirement (`version = "=0.10.0-beta.1`)
  * Macro crate dependencies always use an exact version requirement, even for
    final releases.
  * Otherwise, use `version = "x.y.z"`.

Update the `CHANGELOG.md` of the relevant package.

* If there is already a section for the version to be released, remove the
  `(unreleased)` from its title.
  * Otherwise, change the `[unreleased]` section title to the version being
    released.

Finally, commit these changes as `Release {crate} {version}`.

## Publishing to crates.io and creating a release tag

For `ruma-identifiers-validation` or one of the macro crates, only publish them
to crates.io without creating a release tag:

```
cargo publish
```

For all others, the corresponding `xtask` command does both:

```
cargo xtask release {crate}
```

## Dependencies

Dependencies obviously need to be released before dependents. Also, a breaking
change release in a dependency should usually be followed by a new release of
all dependents.

![crate dependencies](./workspace_deps.png)

<small><code>cargo depgraph --all-features --exclude syn,quote,js_int,trybuild,criterion,proc-macro2,serde,http,form_urlencoded,serde_json,proc-macro-crate,rand,either,ring,base64,itoa,untrusted,futures-util,hyper,hyper-tls,hyper-rustls,thiserror,paste,assign,futures-core,maplit,percent-encoding --dedup-transitive-deps | dot -Tpng</code></small>
