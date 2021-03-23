# ruma-serde

This crate contains (de)serialization helpers for other ruma crates.

Part of that is a fork of [serde_urlencoded], with support for sequences in `Deserialize` /
`Serialize` structs (e.g. `Vec<Something>`) that are (de)serialized as `field=val1&field=val2`.

[serde_urlencoded]: https://github.com/nox/serde_urlencoded
