# [unreleased]

Breaking Changes:

* Replace `RoomV3Pdu` with `ruma_events::pdu::{Pdu, PduStub}`.

Improvements:

* Add endpoints:
    ```
    authorization::get_event_authorization::v1,
    directory::get_public_rooms::v1,
    discovery::{
        discover_homeserver,
        get_server_keys::v2,
        get_server_version::v1
    },
    membership::{
        create_join_event::{v1, v2},
        create_join_event_template::v1
    },
    openid::{
        get_openid_userinfo::v1
    },
    query::{
      get_profile_information::v1,
      get_room_information::v1,
    },
    transactions::send_transaction_message::v1,
    version::get_server_version::v1
    ```

# 0.0.1

Improvements:

* Provide `RoomV3Pdu` type for room versions 3 and above
