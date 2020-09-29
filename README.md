# Ruma – Your home in Matrix.

A set of [Rust] crates (libraries) for interacting with the [Matrix] chat network.

[website] • [chat]

[Rust]: https://rust-lang.org/
[Matrix]: https://matrix.org/
[website]: https://www.ruma.io/
[chat]: https://matrix.to/#/#ruma:matrix.org

## Status

As of 2020-09-29, we support the vast majority of endpoints in all of the various Matrix APIs.

As long as they are still at version 0.x though, only the latest API revision is considered
supported. This may change after 1.x releases.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## Minimum Rust version

Ruma currently requires Rust 1.43.0. In general, we will never require beta or
nightly for crates.io releases of our crates, and we will try to avoid releasing
crates that depend on features that were only just stabilized.

The exception to this is ruma-signatures (and hence ruma with the federation-api
feature) since it depends on [ring][], which is only guaranteed to work on the
latest stable.

[ring]: https://github.com/briansmith/ring/

## License

[MIT](http://opensource.org/licenses/MIT)
