# [unreleased]

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
