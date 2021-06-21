# How to release new versions of the Ruma crates

Releasing one of the crates is very simple since it is entirely automated.
The only thing you have to do is to run

```
cargo xtask release {crate} {version}
```

The `xtask` script will then take care of

* updating all affected `Cargo.toml`s
* adding a new version header to the changelog
* collecting the changes from the changelog
* creating a release commit
* publishing the new release to [crates.io](https://crates.io/)
* creating a release tag and GitHub release if applicable

If some part of `cargo xtask release` fails, for example because of internet
connectivity issues, you can run the exact same command again to retry. Steps
that were already completed will be detected and an option to continue with
the next step will be given.

## Dependencies

Dependencies obviously need to be released before dependents. Also, a breaking
change release in a dependency should usually be followed by a new release of
all dependents.

![crate dependencies](./workspace_deps.png)

<small><code>cargo depgraph --all-features --exclude syn,quote,js_int,trybuild,criterion,proc-macro2,serde,http,serde_json,proc-macro-crate,rand,either,base64,itoa,hyper,hyper-tls,hyper-rustls,thiserror,paste,assign,maplit,percent-encoding,xtask,itertools,isahc,reqwest,async-stream,async-trait,futures-core,futures-lite,pkcs8,ed25519-dalek,pulldown-cmark,tracing,indoc,wildmatch,indexmap,sha2,bytes,form_urlencoded --dedup-transitive-deps | dot -Tpng</code></small>
