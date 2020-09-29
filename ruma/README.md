# ruma

[![crates.io page](https://img.shields.io/crates/v/ruma.svg)](https://crates.io/crates/ruma)
[![docs.rs page](https://docs.rs/ruma/badge.svg)](https://docs.rs/ruma/)
![license: MIT](https://img.shields.io/crates/l/ruma.svg)

Types and traits for working with the Matrix protocol.

This crate re-exports things from all of the other ruma crates so you don't
have to manually keep all the versions in sync.

Which crates are re-exported can be configured through cargo features.
Depending on which parts of Matrix are relevant to you, activate the
following features:

* `client-api` for the client-server API
* `federation-api` for the server-server (federation) API
* `appservice-api` for the application service API
