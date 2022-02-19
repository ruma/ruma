# [unreleased]

# 0.6.0

Breaking changes:

* Most validation functions no longer return the colon position on success

Improvements:

* Add `mxc_uri` validation

# 0.5.0

Breaking changes:

* Make `Error` type non-exhaustive

# 0.4.0

Breaking changes:

* Fix a typo in a public function name: `user_id::localpart_is_fully_{comforming => conforming}`

# 0.3.0

Breaking changes:

* Remove the `serde` feature

# 0.2.4

Improvements:

* Restore the `serde` feature which was accidentally removed in a patch release

# 0.2.3

Improvements:

* Add a `compat` feature
  * Under this feature, more user IDs are accepted that exist in the while but are not
    spec-compliant

# 0.2.2

Improvements:

* Add verification of `mxc://` URIs

# 0.2.1

Improvements:

* Drop unused dependencies

# 0.2.0

Breaking changes:

* Remove `key_algorithms` module (moved to ruma-identifiers as `crypto_algorithms`)
