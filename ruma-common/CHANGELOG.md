# [unreleased]

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
