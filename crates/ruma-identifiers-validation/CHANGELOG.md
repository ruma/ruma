# [unreleased]

Breaking changes:

- `user_id::validate` is now compatible with all non-compliant user IDs in the
  wild by default, due to a clarification in the spec.
  - The `compat-user-id` cargo feature was removed.
  - `user_id::localpart_is_backwards_compatible` can be used to validate the
    localpart of user IDs received over federation.
  - `user_id::localpart_is_fully_conforming` always strictly validates against
    the strict and historical grammars, regardless of the compat features that
    are enabled.
  - `user_id::validate_strict` allows to validate strictly a user ID against the
    strict grammar, regardless of the compat features that are enabled.

Bug fixes:

- Don't allow alphanumeric characters outside of the ASCII range in
  `room_version_id::validate()`. It used to allow any Unicode alphanumeric
  character.

Improvements:

- The maximum allowed length of Matrix identifiers is exposed as `ID_MAX_BYTES`.
- `room_id::validate` disallows the `NUL` byte, due to a clarification in the
  spec.
- `room_alias_id::validate` disallows the `NUL` byte for the localpart, due to a
  clarification in the spec.
- Add `space_child_order::validate`.

# 0.10.1

Improvements:

- Upgrade `thiserror` to 2.0.0.

# 0.10.0

Breaking changes:

- `key_id::validate` takes a generic parameter that implements the new `KeyName`
  trait to validate the key name part. This allows to validate key names that
  are not only server signing key versions.
- The `compat-key-id` cargo feature was renamed to
  `compat-server-signing-key-version`.
- Remove the `device_key_id` module. `DeviceKeyId` is now validated with
  `key_id::validate`.

Improvements:

- Add `server_signing_key_version::validate`.
- Add `base64_public_key::validate`.

# 0.9.5

Bug fixes:

- Allow underscores (`_`) when validating MXC URIs.
  - They have always been allowed in the spec in order to support URL-safe
    base64-encoded media IDs.

Improvements:

- Point links to the Matrix 1.10 specification

# 0.9.4

Yanked because it was created from the wrong branch

# 0.9.3

Improvements:

- Don't require room IDs to contain a server name
  - Room IDs being splittable into localpart and servername does not have
    much inherent value and there are proposals like [MSC4051] that propose
    changing the format. Relaxing the rules makes Ruma forwards-compatible
    with those proposals. The server_name accessor is kept because it is
    used by at least one downstream, but is updated to return an `Option`.

[MSC4051]: https://github.com/matrix-org/matrix-spec-proposals/pull/4051

# 0.9.2

Bug fixes:

- Don't consider empty localpart of a user ID as valid
  - It is accepted under the `compat-user-id` feature, but not considered
    fully-conforming

Improvements:

- Allow `+` in the localpart of user IDs according to MSC4009 / Matrix 1.8
- Add `compat-arbitrary-length-ids` Cargo feature for opting out of 255-byte
  length check for all ID types

# 0.9.1

Improvements:

* Update links to the latest version of the Matrix spec

# 0.9.0

Breaking changes:

* Remove `room_name` module
  * Room name size limits were never enforced, so they are now just regular
    `String`s in Ruma ([Spec change removing the size limit][spec])

[spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/3669

# 0.8.1

Improvements:

* Remove unused dependency on `url`

# 0.8.0

Breaking changes:

* Rework the `Error` type (merge / rename variants)

# 0.7.0

Improvements:

* Add more `Error` variants

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

* Fix a typo in a public function name: `user_id::localpart_is_fully_conforming`

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
