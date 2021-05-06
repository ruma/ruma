# Ruma – Your home in Matrix.

A set of [Rust] crates (libraries) for interacting with the [Matrix] chat
network.

[website] • [chat] • [unstable documentation][docs]

[Rust]: https://rust-lang.org/
[Matrix]: https://matrix.org/
[website]: https://www.ruma.io/
[chat]: https://matrix.to/#/#ruma:matrix.org
[docs]: https://docs.ruma.io/

## Getting started

If you want to build a Matrix client or bot, have a look at [matrix-rust-sdk].
It builds on Ruma and includes handling of state storage, end-to-end encryption
and many other useful things.

For homeservers, bridges and harder-to-categorize software that works with
Matrix, you're at the right place. To get started, add `ruma` to your
dependencies (as a git dependency if you want all of the latest improvements).

`ruma` re-exports all of the other crates, so you don't have to worry about
them. Check out [docs.ruma.io](https://docs.ruma.io/ruma/index.html) for the
latest documentation including which Cargo features you have to enable for the
functionality you want. If you are using a released version from crates.io, you
can also find versioned documentation [on docs.rs](https://docs.rs/ruma/).

[matrix-rust-sdk]: https://github.com/matrix-org/matrix-rust-sdk#readme
[feat]: https://github.com/ruma/ruma/blob/1166af5a354210dcbced1eaf4a11f795c381d2ec/ruma/Cargo.toml#L35

## Status

As of 2021-05-06, we support the vast majority of endpoints in all of the
various Matrix APIs, the notable exception being the identity service API,
where a few endpoints are still missing.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Minimum Rust version

Ruma currently requires Rust 1.45. In general, we will never require beta or
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
