# Ruma – Your home in Matrix.

A set of [Rust] crates (libraries) for interacting with the [Matrix] chat network.

[website] • [chat]

[Rust]: https://rust-lang.org/
[Matrix]: https://matrix.org/
[website]: https://www.ruma.io/
[chat]: https://matrix.to/#/#ruma:matrix.org

## Getting started

If you want to build a Matrix client or bot, have a look at [matrix-rust-sdk].
It builds on Ruma and includes handling of state storage, end-to-end encryption
and many other useful things.

For homeservers, bridges and harder-to-categorize software that works with
Matrix, you're at the right place. To get started, add `ruma` to your
dependencies (as a git dependency if you want all of the latest improvements),
enable all [Cargo features][feat] that seem relevant and run
`cargo doc -p ruma --open`. The `ruma` crate re-exports all relevant things,
except for `ruma-client`, which you can use to make client-API calls to a Matrix
homeserver¹.

If you use the crates through crates.io, make sure to choose versions of `ruma`
and `ruma-client` that depend on the same / compatible versions of the other
crates (checking one, for example `ruma-common`, is enough though). At the time
of writing the latest versions are `ruma 0.0.2` and `ruma-client 0.5.0-alpha.1`,
which can be used together.

If you're using the crates through git, just use the same `rev` (or `branch` if
you want to control the exact version only through `Cargo.lock`) for both.

This may seem a little convoluted, that's because it is. We're working on it.

¹ (better) support for the other APIs is planned

[matrix-rust-sdk]: https://github.com/matrix-org/matrix-rust-sdk#readme
[feat]: https://github.com/ruma/ruma/blob/1166af5a354210dcbced1eaf4a11f795c381d2ec/ruma/Cargo.toml#L35

## Status

As of 2021-01-19, we support the vast majority of endpoints in all of the various Matrix APIs
except the identity service API (if you want to help with that, have a look at the
[crate/ruma-identity-service-api][id-api] label).

[id-api]: https://github.com/ruma/ruma/issues?q=is%3Aissue+is%3Aopen+label%3Acrate%2Fruma-identity-service-api

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Minimum Rust version

Ruma currently requires Rust 1.43. In general, we will never require beta or
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
