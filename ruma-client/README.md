# ruma-client

[![crates.io page](https://img.shields.io/crates/v/ruma-client.svg)](https://crates.io/crates/ruma-client)
[![docs.rs page](https://docs.rs/ruma-client/badge.svg)](https://docs.rs/ruma-client/)
[![build status](https://travis-ci.org/ruma/ruma-client.svg?branch=master)](https://travis-ci.org/ruma/ruma-client)
![license: MIT](https://img.shields.io/crates/l/ruma-client.svg)

**ruma-client** is a [Matrix][] client library for [Rust][].

[Matrix]: https://matrix.org/
[Rust]: https://www.rust-lang.org/

## Status

This project is a work in progress and not ready for production usage yet. Most endpoints that are
available in this crate are usable with an up-to-date synapse server, but no comprehensive testing
has been done so far.

As long as the matrix client-server API is still at version 0.x, only the latest API revision is
considered supported. However, due to the low amount of available manpower, it can take a long time
for all changes from a new API revision to arrive in ruma-client (at the time of writing only few
endpoints have received an update for r0.4.0).

## Contributing

If you want to help out, have a look at the issues here and on the other [ruma-\*][gh-org]
repositories (ruma-client-api and ruma-events in particular contain much of the code that powers
ruma-client).

There is also a [room for ruma on matrix.org][#ruma:matrix.org], which can be used for questions
and discussion related to any of the crates in this project.

[gh-org]: https://github.com/ruma
[#ruma:matrix.org]: https://matrix.to/#/#ruma:matrix.org

## Minimum Rust version

ruma-client requires Rust 1.39.0 or later.
