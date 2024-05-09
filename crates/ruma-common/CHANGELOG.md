# [unreleased]

# 0.13.0

Bug fixes:

- Allow to deserialize `Ruleset` with missing fields.

Breaking changes:

- The power levels fields in `PushConditionRoomCtx` are grouped in an optional `power_levels` field.
  If the field is missing, push rules that depend on it will never match. However, this allows to
  match the `.m.rule.invite_for_me` push rule because usually the `invite_state` doesn't include
  `m.room.power_levels`.
- Add support for endpoints that take an optional authentication
- Add support for endpoints that require authentication for appservices
- `deserialize_as_f64_or_string` has been extended to also support parsing integers, and renamed to
  `deserialize_as_number_or_string` to reflect that.
- The http crate had a major version bump to version 1.1
- `IntoHttpError::Header` now contains a `HeaderSerializationError`

Improvements:

- Use the [web-time](https://github.com/daxpedda/web-time) crate to return a
  `SystemTime` that works under WASM in the
  `MilliSecondsSinceUnixEpoch::to_system_time()` method.
- Stabilize support for `.m.rule.suppress_edits` push rule (MSC3958 / Matrix 1.9)
- Add `MatrixVersion::V1_9` and `V1_10`
- Point links to the Matrix 1.10 specification
- Implement `as_str()` and `AsRef<str>` for `push::PredefinedRuleId`
- Implement `kind()` for `push::Predefined{*}RuleId`
- Implement `Clone` for `MatrixToUri` and `MatrixUri`

# 0.12.1

Bug fixes:

- Allow to deserialize `(New)ConditionalPushRule` with a missing `conditions` field.

# 0.12.0

Bug fixes:

- Set the predefined server-default `.m.rule.tombstone` push rule as enabled by default, as defined
  in the spec.

Breaking changes:

- `FlattenedJson::get` returns a `FlattenedJsonValue` instead of a string
- Remove the `DontNotify` and `Coalesce` variants of `push::Action` according to MSC3987
  - Old push rules will still deserialize successfully but the `Coalesce` variant will not return
    `true` for `Action::should_notify()` anymore
- Removed the `events` module, it is once again its own crate (`ruma-events`)
- Removed `From` and `TryFrom` implementations for `RedactedBecause` in favor of named constructors
  (`from_json` and `from_raw_event`)
- Updated room IDs to not require a servername
  - Removed `localpart` method from `RoomId` and `RoomOrAliasId`
  - Changed `server_name` method on `RoomId` and `RoomOrAliasId` to return `Option<&str>`

Improvements:

- Allow padding when decoding the `Base64` type from a string
- Add convenience methods for `push::Ruleset`:
  - To update the server-default push rules
  - To remove a user-defined push rule
- Add `AsRef<[u8]>` implementations for identifier types
- `PushCondition::EventMatch` and `FlattenedJson` now use escaped dotted paths (MSC3873 / Matrix 1.7)
- Add support for `event_property_is` push condition (MSC3758 / Matrix 1.7)
- Add support for `event_property_contains` push condition (MSC3966 / Matrix 1.7)
- Add `MatrixVersion::V1_7` and `MatrixVersion::V1_8`
- Add support for room version 11 according to MSC3820 / Matrix 1.8
  - Adapt the redaction algorithm in `canonical_json`
- Add unstable support for suppress edits push rule (MSC3958)

# 0.11.3

Bug fixes:

- Move `.m.rule.roomnotif` push rule before `.m.rule.tombstone` in the server default push rules,
  according to a spec clarification in Matrix 1.6

Improvements:

* Add `MatrixVersion::V1_6`
* Stabilize support for fixed base64 for SAS verification (MSC3783 / Matrix 1.6)
  * Deprecate `MessageAuthenticationCode::HkdfHmacSha256`

# 0.11.2

Bug fixes:

- Don't accept colons in the localpart given to `UserId::parse_with_servername`
  even with `feature = "compat"`

Improvements:

- Derive `Hash` for `ReceiptType` and `ReceiptThread`
- Update `EventContent` derive macro to emit new type definitions and type
  aliases under the same visibility as the input type (this fixes a future-
  compatibility warning when deriving `EventContent` on a non-`pub` type)

# 0.11.1

Improvements:

- Make alternate Debug representation of `MilliSecondsSinceUnixEpoch` and
  `SecondsSinceUnixEpoch` more compact (remove newlines)

# 0.11.0

Bug fixes:

* HTML-relevant characters (`<`, `>`, etc) in plaintext replies are now escaped
  during creation of the rich reply
* Don't include sensitive information in `Debug`-format of types from the `events::key`
  and `events::secret` modules
* Fix deserialization of `RoomMessageEventContent` and `RoomEncryptedEventContent` when there
  is no relation
* Fix deserialization of `StateUnsigned` when the `prev_content` is redacted
* Allow to deserialize `PushCondition` with unknown kind
* Allow to deserialize `push::Action` with unknown value
* Only percent-encode reserved characters in endpoint URL path

Breaking changes:

* Remove deprecated `EventType` enum
* Remove deprecated constructors for `RoomMessageEventContent`
* Remove `serde::vec_as_map_of_empty` from the public API
* Remove the `api::AuthScheme::QueryOnlyAccessToken` variant, which is no longer used
* The `#[ruma_api(header)]` attribute of the `ruma_api` macro now accepts an arbitrary
  `http::header::HeaderName`
  * To continue using constants from `http::header`, they must be imported in
    the module calling the macro.
* Make `name` optional on `SecretStorageKeyEventContent`. Default constructor has been
  adjusted as well to not require this field.
* Rename `push::PusherData` to `HttpPusherData` and make the `url` field required
* Remove `Ruleset::add` and the implementation of `Extend<AnyPushRule>` for `Ruleset`
* Make `EndpointError` construction infallible
  * `EndpointError::try_from_http_request` has been replaced by `EndpointError::from_http_request`
  * `FromHttpResponseError<E>::Server` now contains `E` instead of `ServerError<E>`
  * `ServerError<E>` has been removed
  * `MatrixError` is now an enum with the `Json` variant containing the previous fields
* Change the `ignored_users` field of `IgnoredUserListEventContent` to a map of empty structs, to
  allow eventual fields to be added, as intended by the spec
* Make `SimplePushRule` and associated types generic over the expected type of the `rule_id`
* Deduplicate and group relation structs in `events::relation`:
  * Move relation structs under `events::room::message` to `events::relation`
  * Move common relation structs under `events::room::encrypted` to `events::relation` and remove
    duplicate types
  * Remove `events::reaction::Relation` and use `events::relation::Annotation` instead
  * Remove `events::key::verification::Relation` and use `events::relation::Reference` instead
* Rename `events::relation::Relations` to `BundledRelations`
* Make the `redacted_because` field in `UnsignedRedacted` non-optional and replace parameterless
  `new` constructor by one that takes a redaction event (like `new_because` previously, which is
  now removed)
* Move the `Unsigned` associated type from `StateEventContent` to `OriginalStateEventContent`
  * `Redacted*EventContent`s don't have an `unsigned` type anymore
* Remove the `serde::urlencoded` module
  * Query string (de)serialization is now done by the `serde_html_form` crate
* Rename `RoomEventType` to `TimelineEventType`
* Remove `SecretStorageKeyEventContent`'s implementation of `Deserialize`
  * Use `EventContentFromType::from_parts` instead
* Remove `StateUnsignedFromParts`
  * Replace it with a bound on `DeserializeOwned`
* Remove `Raw::deserialize_content`
  * Instead, use `.deserialize_as::<T>()` or `.cast_ref::<T>().deserialize_with_type()`
* Remove `EventContent::from_parts`
  * Replace it with `EventContentFromType::from_parts`
* The `serde::StringEnum` derive now also generates a `Debug` implementation

Improvements:

* Add `MatrixVersion::V1_4` and `MatrixVersion::V1_5`
* Stabilize default room server ACL push rule
* Stabilize `room_types` in `directory::Filter` and `room_type` in `directory::PublicRoomsChunk`
* Stabilize support for private read receipts
* Add stable support for threads
  * Move `Relation::Thread` and associated types and methods out of `unstable-msc3440`
  * Add parameter to `RoomMessageEventContent::make_reply_to` to be thread-aware
  * Add `RoomMessageEventContent::make_for_reply`
* Stabilize support for event replacements (edits)
* Add support for read receipts for threads (MSC3771 / Matrix 1.4)
* Add `push::PredefinedRuleId` and associated types as a list of predefined push rule IDs
* Add convenience methods to `Ruleset`
  * `Ruleset::get` to access a push rule
  * `Ruleset::insert` to add or update user push rules
  * `Ruleset::set_enabled` to change the enabled state of push rules
  * `Ruleset::set_actions` to change the actions of push rules
* Add support for bundled reference relations (MSC3267 / Matrix 1.5)
* Add the `formatted` field on `KeyVerificationRequestEventContent` (Matrix 1.5)
* Add `content` accessors for `Any*StateEvent` enums
* Add the `DebugAsRefStr` derive macro to `ruma_common::serde`

# 0.10.5

Improvements:

* Add support for `#[incoming_derive(!Debug)]` to the `Incoming` derive macro

# 0.10.4

Bug fixes:

* Fix `MatrixToUri` parsing for non-url-encoded room aliases

# 0.10.3

Bug fixes:

* Fix ruma-common not compiling with the Cargo features `events` and
  `unstable-msc2677` active, and `unstable-msc2676` inactive

# 0.10.2

Improvements:

* Add `relations` accessors to event enum types:
  * `AnyMessageLikeEvent` and `AnySyncMessageLikeEvent`
  * `AnyStateEvent` and `AnySyncStateEvent`
  * `AnyTimelineEvent` and `AnySyncTimelineEvent`

# 0.10.1

Improvements:

* Add `RoomMessageEventContent::make_reply_to`
  * Deprecate reply constructors in favor of the new method

# 0.10.0

Bug fixes:

* Expose `MatrixIdError`, `MatrixToError`, `MatrixUriError` and `MxcUriError` at
  the crate root
* Fix matching of `event_match` condition
  * The spec clarified its behavior:
    <https://github.com/matrix-org/matrix-spec-proposals/pull/3690> 

Breaking changes:

* Add `user_id` field to `PushConditionRoomCtx`
* Remove `PartialEq` implementation on `NotificationPowerLevels`
* Remove `PartialEq` implementation for `events::call::SessionDescription`
* Use new `events::call::AnswerSessionDescription` for `CallAnswerEventContent` 
  and `OfferSessionDescription` for `CallInviteEventContent`
* Use new `VoipVersionId` and `VoipId` types for `events::call` events
* Remove `RoomName` / `OwnedRoomName` and replace usages with `str` / `String`
  * Room name size limits were never enforced by servers
    ([Spec change removing the size limit][spec])
* Remove `RoomMessageFeedbackEvent` and associated types and variants according to MSC3582
* Move `CanonicalJson`, `CanonicalJsonObject` and `CanonicalJsonError` out of
  the `serde` module and behind the cargo feature flag `canonical-json`
* Make identifiers matrix URI constructors generic over owned parameters
  * Split `RoomId` matrix URI constructors between methods with and without routing
* Allow to add routing servers to `RoomId::matrix_to_event_uri()`
* Move `receipt::ReceiptType` to `events::receipt`
* Make `Clone` as supertrait of `api::OutgoingRequest`
* Rename `Any[Sync]RoomEvent` to `Any[Sync]TimelineEvent`
* `RoomMemberEvent` and related types now have a custom unsigned type including the
  `invite_room_state` field, instead of the `StateUnsigned` type used by other state
  events

[spec]: https://github.com/matrix-org/matrix-spec-proposals/pull/3669

Improvements:

* All push rules are now considered to not apply to events sent by the user themselves
* Change `events::relation::BundledAnnotation` to a struct instead of an enum
  * Remove `BundledReaction`
* Add unstable support for polls (MSC3381)
* Add unstable support for Improved Signalling for 1:1 VoIP (MSC2746)
* Add support for knocking in `events::room::member::MembershipChange`
* Add `MatrixVersion::V1_3`
* Deprecate the `sender_key` and `device_id` fields for encrypted events (MSC3700)
* Move the `relations` field of `events::unsigned` types out of `unstable-msc2675`
* Deserialize stringified integers for power levels without the `compat` feature
* Add `JoinRule::KnockRestricted` (MSC3787)
* Add `MatrixVersionId::V10` (MSC3604)
* Add methods to sanitize messages according to the spec behind the `html` feature
  * Can also remove rich reply fallbacks
* Implement `From<Owned*Id>` for `identifiers::matrix_uri::MatrixId`
* Add unstable default push rule to ignore room server ACLs events (MSC3786)
* Add unstable support for private read receipts (MSC2285)
* Add unstable support for filtering public rooms by room type (MSC3827)

# 0.9.2

Bug fixes:

* Fix serialization and deserialization of events with a dynamic `event_type`

# 0.9.1

Improvements:

* Add `StrippedPowerLevelsEvent::power_levels`
* Add (`Sync`)`RoomMemberEvent::membership`
* Export `events::room::member::Change`
  * Prior to this, you couldn't actually do anything with the
    `membership_change` functions on various member event types

# 0.9.0

Bug fixes:

* Change default `invite` power level to `0`
  * The spec was determined to be wrong about the default:
    <https://github.com/matrix-org/matrix-spec/pull/1021>

Breaking changes:

* Several ruma crates have been merged into `ruma-common`
  * `ruma-api` has moved into `api`, behind a feature flag
  * `ruma-events` has moved into `events`, behind a feature flag
  * `ruma-identifiers` types are available at the root of the crate
  * `ruma-serde` has moved into `serde`
* The `events::*MessageEvent` types have been renamed to `*MessageLikeEvent`
* Change `events::room` media types to accept either a plain file or an
  encrypted file, not both simultaneously
* Change `events::room` media types to use `Duration` where applicable
* Move `prev_content` into `unsigned`
* Rename `identifiers::Error` to `IdParseError`
* Fix the `RoomMessageEventContent::*_reply_plain` methods that now return a
  message with a `formatted_body`, according to the spec. Therefore, they only
  accept `OriginalRoomMessageEvent`s like their HTML counterparts.
* Update the `state_key` field of state events to be of a different type
  depending on the content type. You now no longer need to validate manually
  that `m.room.member` events have a user ID as their state key!

Improvements:

* Add unstable support for extensible events (MSCs 1767, 3551, 3552, 3553, 3246, 3488)
* Add unstable support for translatable text content (MSC3554)
* Add unstable support for voice messages (MSC3245)
* Add unstable support for threads (MSC3440)
* Add `ReceiptEventContent::user_receipt`
* Make `Restricted::allow` public
* Conversion from `RoomPowerLevels` to `RoomPowerLevelsEventContent`

# 0.8.0

Breaking changes:

* Update `ruma-identifiers` dependency

# 0.7.0

Breaking changes:

* Update `ruma-identifiers` dependency
* Use new `Base64` type for `key` field of `SignedKey`

# 0.6.0

Breaking changes:

* Make a few enums non-exhaustive
* Upgrade dependencies

# 0.5.4

Improvements:

* Add `to_device` module containing `DeviceIdOrAllDevices`

# 0.5.3

Improvements:

* Add `instance_id` field to `ProtocolInstance[Init]` under the
  `unstable-pre-spec` feature

# 0.5.2

Improvements:

* Add `thirdparty::ThirdPartyIdentifier`

# 0.5.1

Improvements:

* Add `receipt::ReceiptType`
* Add `MilliSecondsSinceUnixEpoch` and `SecondsSinceUnixEpoch` types
* Bump dependency versions

# 0.5.0

Breaking changes:

* Rename `push::RulesetIter` to `push::RulesetIntoIter`
* Change the return type of `push::Ruleset::get_actions` from an iterator to a
  slice

Improvements:

* Add `push::Ruleset::iter()` for borrowing iteration of rulesets
* Add conversions between `AnyPushRule` and `AnyPushRuleRef`
  (`AnyPushRule::as_ref` and `AnyPushRuleRef::to_owned`)
* Add `push::Ruleset::get_match()` for finding the first matching push rule for
  an event. This is pretty much the same thing as `get_actions()` but returns
  the entire push rule, not just its actions.

# 0.4.0

Breaking changes:

* Use `ruma_identifiers::MxcUri` instead of `String` for `avatar_url` field in
  `directory::PublicRoomsChunk`
* Use `ruma_identifiers::RoomId` instead of `String` for `room_id` field in
  `push::PushConditionRoomCtx`
* Upgrade ruma-identifiers dependency to 0.19.0

# 0.3.1

Bug fixes:

* Fix `push::PushCondition::applies` for empty value and pattern

# 0.3.0

Breaking changes:

* Update set of conversion trait implementations for enums
* Replace `Vec` by `IndexSet` in `push::Ruleset`
* Replace `push::AnyPushRule` with an enum (the original struct still exists as
  just `PushRule` in `ruma-client-api`)
* … (there's a lot more, but this changelog was not kept up to date; PRs to
  improve it are welcome)

Improvements:

* Add the `thirdparty` module
* Add `directory::{Filter, PublicRoomsChunk, RoomNetwork}` (moved from
  `ruma_client_api::r0::directory`)
* Add `push::{PusherData, PushFormat}` (moved from `ruma_client_api::r0::push`)
* Add `authentication::TokenType` (moved from
  `ruma_client_api::r0::account:request_openid_token`)
* Add an `IntoIterator` implementation for `Ruleset`
* Add `push::Ruleset::get_actions`
  * Add `push::PushCondition::applies`
  * Add `push::{FlattenedJson, PushConditionRoomCtx}`

# 0.2.0

Breaking changes:

* Make most types defined by the crate `#[non_exhaustive]`
