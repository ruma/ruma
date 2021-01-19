# Ruma – Your home in Matrix.

A set of [Rust] crates (libraries) for interacting with the [Matrix] chat network.

[website] • [chat]

[Rust]: https://rust-lang.org/
[Matrix]: https://matrix.org/
[website]: https://www.ruma.io/
[chat]: https://matrix.to/#/#ruma:matrix.org

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
