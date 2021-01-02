# How to release new versions of the Ruma crates

## General

* When releasing a pre-release version, make sure other crates are updated to
  depend on it, with an exact version requirement (`=x.y.z-tag`)
* When releasing a regular version, make sure other crates are updated to depend
  on it, with a regular (same as caret / `^`) version requirement (`x.y.z`)
* Macro crates are versioned identically to their "parent" crate and are exempt
  from the rule above: Dependencies from the parent crate to the macro crate
  (for re-exporting) should always use exact version requirements. Whenever a
  crate with an associated macro crate should get a new release, release a new
  version of the macro crate with the same number first, even if there were no
  changes in the macro code.

## Dependencies

Dependencies obviously need to be released before dependents. Also, a breaking
change release in a dependency should usually be followed by a new release of
all dependents.

![crate dependencies](./workspace_deps.png)

<small><code>cargo depgraph --all-features --exclude syn,quote,js_int,trybuild,criterion,proc-macro2,serde,http,form_urlencoded,serde_json,proc-macro-crate,rand,either,ring,base64,itoa,untrusted,futures-util,hyper,hyper-tls,hyper-rustls,thiserror,paste,assign,futures-core,maplit,percent-encoding --dedup-transitive-deps | dot -Tpng</code></small>
