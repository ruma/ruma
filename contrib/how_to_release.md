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

<small><code>cargo depgraph --all-features --exclude anyhow,assign,async-stream,async-trait,base64,bytes,criterion,ed25519-dalek,either,form_urlencoded,futures-core,futures-lite,getrandom,joke_bot,js_int,js_option,hello_isahc,hello_world,http,hyper,hyper-rustls,hyper-tls,indexmap,indoc,isahc,itertools,itoa,maplit,message_log,percent-encoding,pkcs8,proc-macro2,proc-macro-crate,pulldown-cmark,quote,rand,reqwest,ryu,serde,serde_json,sha2,syn,thiserror,tokio,tokio-stream,tracing,url,uuid,wildmatch,xflags,xshell,xtask --dedup-transitive-deps | dot -Tpng > contrib/workspace_deps.png</code></small>
