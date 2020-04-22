# [unreleased]

Improvements:

* Add a encrypted variant to the room `MessageEventContent` enum.

# 0.20.0

Improvements:

* Update ruma-identifiers to 0.16.0

# 0.19.0

Breaking changes:

* Update ruma-identifiers to 0.15.1
* Change timestamps, including `origin_server_rs` from `UInt` to `SystemTime`
* Change all usages of `HashMap` to `BTreeMap`
  * To support this, `EventType` now implements `PartialOrd` and `Ord`

# 0.18.0

Breaking changes:

* Update unsigned field's type from `Option<Value>` to `Map<String, Value>`

Improvements:

* Add a convenience constructor to create a plain-text `TextMessageEventContent`
* Add `m.dummy` events to the to-device event collection

# 0.17.0

Breaking changes:

* `collections::only` no longer exports a `raw` submodule. It was never meant ot be exported in the first place.
* Renamed `stripped::{StrippedState => AnyStrippedStateEvent, StrippedStateContent => StrippedStateEvent}`

Improvements:

* Added `to_device` module with to-device variants of events (as found in the `to_device` section of a sync response)
* Added a helper method for computing the membership change from a `MemberEvent`

Bug fixes:

* Fixed missing `m.` in `m.relates_to` field of room messages
* Fixed (de)serialization of encrypted events using `m.olm.v1.curve25519-aes-sha2`

# 0.16.0

Breaking changes:

* `TryFromRaw::try_from_raw`'s signature has been simplified. The previous signature was a relict that was no longer sensible.
* All remaining non-optional `room_id` event fields (not event content fields) have been made optional

Improvements:

* `NameEvent`s are now validated properly and will be rejected if the `name` field is longer than 255 bytes.

# 0.15.1

Bug fixes:

* Deserialization of custom events as part of the types from `ruma_events::collections::{all, only}` was implemented (this was missing after the big fallible deserializion rewrite in 0.15.0)

# 0.15.0

Improvements:

* `ruma-events` now exports a new type, `EventResult`
  * For any event or event content type `T` inside a larger type that should support deserialization you can use `EventResult<T>` instead
  * Conceptually, it is the same as `Result<T, InvalidEvent>`
  * `InvalidEvent` can represent either a deserialization error (the event's structure did not match) or a validation error (some additional constraints defined in the matrix spec were violated)
    * It also contains the original value that was attempted to be deserialized into `T` in `serde_json::Value` form

Breaking changes:

* The `FromStr` implementations for event types were removed (they were the previous implementation of fallible deserialization, but were never integrated in ruma-client-api because they didn't interoperate well with serde derives)

# 0.14.0

Breaking changes:

* Updated to ruma-identifiers 0.14.0.

Improvements:

* ruma-events is now checked against the RustSec advisory database.

# 0.13.0

Breaking changes:

* Events and their content types no longer implement `Deserialize` and instead implement `FromStr` and `TryFrom<&str>`, which take a `&str` of JSON data and return a new `InvalidEvent` type on error.
* Integers are now represented using the `Int` and `UInt` types from the `js_int` crate to ensure they are within the JavaScript-interoperable range mandated by the Matrix specification.
* Some event types have new fields or new default values for previous fields to bring them up to date with version r0.5.0 of the client-server specification.
* Some event types no longer have public fields and instead use a constructor function to perform validations not represented by the type system.
* All enums now include a "nonexhaustive" variant to prevent exhaustive pattern matching. This will change to use the `#[nonexhaustive]` attribute when it is stabilized.
* `ParseError` has been renamed `FromStrError`.

New features:

* This release brings ruma-events completely up to date with version r0.5.0 of the client-server specification. All previously supported events have been updated as necessary and the following events have newly added support:
    * m.dummy
    * m.forwarded_room_key
    * m.fully_read
    * m.ignored_user_list
    * m.key.verification.accept
    * m.key.verification.cancel
    * m.key.verification.key
    * m.key.verification.mac
    * m.key.verification.request
    * m.key.verification.start
    * m.push_rules
    * m.key.encrypted
    * m.key.encryption
    * m.key.server_acl
    * m.key.tombstone
    * m.room_key
    * m.room_key_request
    * m.sticker

Improvements:

* Improved documentation for the crate and for many types.
* Added many new tests.
* rustfmt and clippy are now used to ensure consistent formatting and improved code quality.

# 0.12.0

Improvements:

* ruma-events now runs on stable Rust, requiring version 1.34 or higher.

Bug fixes:

* `CanonicalAliasEvent` and `NameEvent` now allow content being absent, null, or empty, as per the spec.

# 0.11.1

Breaking changes:

* `RoomId` is now optional in certain places where it may be absent, notably the responses of the `/sync` API endpoint.
* A `sender` field has been added to the `StrippedStateContent` type.

Improvements:

* Depend on serde's derive feature rather than serde_derive directly for simplified imports.
* Update to Rust 2018 idioms.

# 0.11.0

Breaking changes:

* The presence event has been modified to match the latest version of the spec. The spec was corrected to match the behavior of the Synapse homeserver.

Improvements:

* Dependencies have been updated to the latest versions.

# 0.10.0

Breaking changes:

* The `EventType`, and collections enums have new variants to support new events.
* The `extra_content` method has been removed from the Event trait.
* The `user_id` method from the `RoomEvent` trait has been renamed `sender` to match the specification.
* The `origin_server_ts` value is now required for room events and is supported via a new `origin_server_ts` method on the `RoomEvent` trait.
* `MemberEventContent` has a new `is_direct` field.
* `FileMessageEventContent` has a new `filename` field.
* File and thumbnail info have been moved from several message types to dedicated `FileInfo`, `ImageInfo`, and `ThumbnailInfo` types.
* `LocationMessageEventContent` has a new info field.
* `PresenceEventContent`'s `currently_active` field has changed from `bool` to `Option`.
* `TypingEventContent` contains a vector of `UserId`s instead of `EventId`s.
* Height and width fields named `h` and `w` in the spec now use the full names `height` and `width` for their struct field names, but continue to serialize to the single-letter names.

New features:

* ruma-events now supports all events according to r0.3.0 of the Matrix client-server specification.
* Added new event: `m.room.pinned_events`.
* Added new event: `m.direct`.

Bug fixes:

* Several places where struct fields used the wrong key when serialized to JSON have been corrected.
* Fixed grammar issues in documentation.

# 0.9.0

Improvements:

* Added default values for various power level attributes.
* Removed Serde trait bounds on `StrippedStateContent`'s generic parameter.
* Updated to version 0.4 of ruma-signatures.

# 0.8.0

Breaking changes

* Updated serde to the 1.0 series.

# 0.7.0

Bug fixes:

* Make the `federate` field optional when creating a room.

# 0.6.0

Breaking changes:

* Updated ruma-identifiers to the 0.9 series.


# 0.5.0

Breaking changes:

* Updated ruma-identifiers to the 0.8 series.

# 0.4.1

Improvements:

* Relaxed version constraints on dependent crates to allow updating to new patch level versions.

# 0.4.0

Breaking changes:

* Updated serde to the 0.9 series.

The public API remains the same.

# 0.3.0

Improvements:

* `ruma_events::presence::PresenceState` now implements `Display` and `FromStr`.

# 0.2.0

Improvements:

* Added missing "stripped" versions of some state events.
* All "stripped" versions of state events are now serializable.


# 0.1.0

Initial release.
