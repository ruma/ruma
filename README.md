# Ruma – Your home in Matrix.

A set of [Rust] crates (libraries) for interacting with the [Matrix] chat
network.

[website] • [chat] • [documentation][docs] ([unstable][unstable-docs])

[Rust]: https://rust-lang.org/
[Matrix]: https://matrix.org/
[website]: https://www.ruma.io/
[chat]: https://matrix.to/#/#ruma:matrix.org
[docs]: https://docs.rs/ruma/
[unstable-docs]: https://docs.ruma.io/

## Getting started

If you want to build a Matrix client or bot, have a look at [matrix-rust-sdk].
It builds on Ruma and includes handling of state storage, end-to-end encryption
and many other useful things.

For homeservers, bridges and harder-to-categorize software that works with
Matrix, you're at the right place. To get started, add `ruma` to your
dependencies:

```toml
# crates.io release
ruma = { version = "0.1", features = [...] }
# git dependency (the next branch contains some breaking changes main doesn't)
ruma = { git = "https://github.com/ruma/ruma", branch = "next", features = [...] }
```

`ruma` re-exports all of the other crates, so you don't have to worry about
them. Check out the documentation [on docs.rs][docs] (or on
[docs.ruma.io][unstable-docs] if you use use the git dependency).

[matrix-rust-sdk]: https://github.com/matrix-org/matrix-rust-sdk#readme
[feat]: https://github.com/ruma/ruma/blob/1166af5a354210dcbced1eaf4a11f795c381d2ec/ruma/Cargo.toml#L35

## Status

As of 2021-05-06, we support the vast majority of endpoints in all of the
various Matrix APIs, the notable exception being the identity service API,
where a few endpoints are still missing.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Minimum Rust version

Ruma currently requires Rust 1.50. In general, we will never require beta or
nightly for crates.io releases of our crates, and we will try to avoid releasing
crates that depend on features that were only just stabilized.

There are two exceptions to this:

* ruma-signatures (and hence ruma with the federation-api feature) since it
  depends on [ring][], which is only guaranteed to work on the latest stable.
* ruma-client depends on some I/O libraries (and also on ring, conditionally),
  so it is also only guaranteed to work on the latest stable.

[ring]: https://github.com/briansmith/ring/

## License

[MIT](http://opensource.org/licenses/MIT)
