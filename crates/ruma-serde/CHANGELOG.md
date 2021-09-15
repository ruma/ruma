# [unreleased]

Breaking changes:

* Remove `From<T>` implementation for `Raw<T>`
  * Replaced by the new fallible constructor `Raw::new`

# 0.5.0

Breaking changes:

* Make `urlencoded::ser::Error` non-exhaustive
* Remove `to_canonical_json_string` and `CanonicalJsonError::JsonSize`
  * The size check only makes sense for PDUs but canonical JSON objects can be
    used for other things too
  * You can simply use `serde_json::to_string` instead

# 0.4.2

Improvements:

* Make `Raw::deserialize` & `Raw::deserialize_as` more general

# 0.4.1

Improvements:

* Remove unneeded cargo feature from a dependency

# 0.4.0

Breaking changes:

* Remove the `empty` module from the public API
* Remove the `time` module

Improvements:

* Add serialization decorator `none_as_empty_string` to serialize `None`s as empty strings
* Add `PartialOrdAsRefStr`, `OrdAsRefStr` and `PartialEqAsRefStr` derives
* Add `MilliSecondsSinceUnixEpoch` and `SecondsSinceUnixEpoch` types
* Add `Raw::{get_field, deserialize_as}`
* Add accessor methods to `CanonicalJsonValue`
* Add conversion trait implementations for `CanonicalJsonValue`

# 0.3.1

Bug fixes:

* Fix an edge case in query parameter deserialization

# 0.3.0

Breaking changes:

* Upgrade `js_int` to 0.2.0
