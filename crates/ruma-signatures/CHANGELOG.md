# [unreleased]

Bug fixes:

- Do not check the signature of the server of the sender of `m.room.member`
  invite events with a `third_party_invite` field.

# 0.17.0

Improvements:

- The `unstable-exhaustive-types` cargo feature was replaced by the
  `ruma_unstable_exhaustive_types` compile-time `cfg` setting. Like all `cfg`
  settings, it can be enabled at compile-time with the `RUSTFLAGS` environment
  variable, or inside `.cargo/config.toml`. It can also be enabled by setting
  the `RUMA_UNSTABLE_EXHAUSTIVE_TYPES` environment variable.

# 0.16.0

Upgrade `ruma-common` to 0.14.0.

# 0.15.0

No changes for this version

# 0.14.0

Breaking changes:

- Update `ed25519-dalek` crate
  - `Ed25519KeyPair::generate()` returns a `Zeroizing<Vec<u8>>` on success
  - `Ed25519KeyPair::public_key()` returns an array instead of a slice

Bug fixes:

- Ignore keys with unknown algorithms in `verify_events`

Improvements:

- Remove `age_ts` from `REFERENCE_HASH_FIELDS_TO_REMOVE` according to a spec clarification

# 0.13.1

No changes for this version

# 0.13.0

No changes for this version

# 0.12.0

Breaking changes:

* Remove pointless `PartialEq` implementation for `Ed25519Verifier`

# 0.11.0

Breaking changes:

* Upgrade dependencies

# 0.10.0

Breaking changes:

* Merge `SplitError` into `Error`
* Update some function signatures to use the new `Base64` type

Improvements:

* Move Room Version 9 keys out of `unstable-pre-spec` in `allowed_content_keys_for`

# 0.9.0

Breaking changes:

* Change a few functions to return `Result`s
  * See each function's documentation for how it can fail

Bug fixes:

* Don't check stringified JSON size <= 65535 bytes for verify_json and sign_json
  since these functions may be used for things other than PDUs

# 0.8.0

Breaking changes:

* Replace `ring` dependency with `ed25519-dalek` and `pkcs8`
* `canonical_json` and `content_hash` now return `Error` when JSON is not canonical

# 0.7.2

Improvements:

* Add a `compat` feature

  When enabled, ruma-signatures will accept slightly malformed base64 input.

# 0.7.1

Improvements:

* Fix verify_json signature check algorithm
* Bump dependency versions

# 0.7.0

Breaking changes:

* Upgrade ruma-identifiers dependency to 0.19.0

# 0.6.0

Breaking changes:

* Remove `Copy` implementation for `Algorithm`
* Remove `Copy` and `Clone` implementations for `Ed25519Verifier`
* Upgrade ruma-identifiers

Bug fixes:

* Verify only the required signatures on `verify_event`
* Fix redactions for aliases events
