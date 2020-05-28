# [unreleased]

Improvements:

* Add endpoints:
    ```
    directory::get_public_rooms::v1,
    discovery::{
        discover_homeserver,
        get_server_keys::v2,
        get_server_version::v1
    },
    membership::{
        create_join_event::v1,
        create_join_event_template::v1
    },
    query::get_room_information::v1,
    transactions::send_transaction_message::v1,
    version::get_server_version::v1
  ```

# 0.0.1

Improvements:

* Provide `RoomV3Pdu` type for room versions 3 and above