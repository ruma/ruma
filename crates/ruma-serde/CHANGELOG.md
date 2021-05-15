# 0.4.0 (unreleased)

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
