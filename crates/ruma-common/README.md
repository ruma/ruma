# ruma-common

[![crates.io page](https://img.shields.io/crates/v/ruma-common.svg)](https://crates.io/crates/ruma-common)
[![docs.rs page](https://docs.rs/ruma-common/badge.svg)](https://docs.rs/ruma-common/)
![license: MIT](https://img.shields.io/crates/l/ruma-common.svg)

Common types for other Ruma crates.

The feature-gated modules are defined as follow:

### `api` module

Behind the `api` feature, core types used to define the requests and responses for each endpoint in
the various [Matrix](https://matrix.org/) API specifications. These types can be shared by client
and server code for all Matrix APIs.

### `events` module

Behind the `events` feature, serializable types for the events in the [Matrix](https://matrix.org/)
specification that can be shared by client and server code.