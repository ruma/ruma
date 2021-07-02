# [unreleased]

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
