# [unreleased]

Breaking changes:

* Add `user_id` field to `PushConditionRoomCtx`
* Remove `PartialEq` implementation on `NotificationPowerLevels`
* Remove `PartialEq` implementation for `events::call::SessionDescription`
* Split `events::call::SessionDescription` into `AnswerSessionDescription`
  and `OfferSessionDescription`
  * Remove `SessionDescriptionType`

Improvements:

* All push rules are now considered to not apply to events sent by the user themselves
* Change `events::relation::BundledAnnotation` to a struct instead of an enum
  * Remove `BundledReaction`
* Add unstable support for polls (MSC3381)

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
