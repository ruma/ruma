# [unreleased]

Breaking changes:

* Update strum dependency to 0.19

Improvements:

* Add the `thirdparty` module
* Add `directory::{Filter, PublicRoomsChunk, RoomNetwork}` (moved from
  `ruma_client_api::r0::directory`)
* Add `push::{PusherData, PushFormat}` (moved from `ruma_client_api::r0::push`)
* Add `authentication::TokenType` (moved from
  `ruma_client_api::r0::account:request_openid_token`)

# 0.2.0

Breaking changes:

* Make most types defined by the crate `#[non_exhaustive]`
