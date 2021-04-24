# 0.4.0 (unreleased)

Breaking changes:

* Remove the `empty` module from the public API

Improvements:

* Add serialization decorator `none_as_empty_string` to serialize `None`s as empty strings
* Add unit tests for `empty_string_as_none` and `none_as_empty_string`

# 0.3.1

Bug fixes:

* Fix an edge case in query parameter deserialization

# 0.3.0

Breaking changes:

* Upgrade `js_int` to 0.2.0
