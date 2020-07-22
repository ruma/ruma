# ruma-serde

This crate contains (de)serialization helpers for other ruma crates.

Part of that is a fork of serde_urlencoded, with support for sequences in `Deserialize` /
`Serialize` structs (e.g. `Vec<Something>`) that are (de)serialized as `field=val1&field=val2`
(instead of the more common `field[]=val1&field[]=val2` format supported by other crates like
`serde_qs`).
