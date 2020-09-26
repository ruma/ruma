# Ruma – Your home in Matrix.

*Monorepo for our various Rust + Matrix crates.*

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
