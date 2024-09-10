fn main() {
    _ = ruma_common::device_key_id!("ed25519:JLAFKJWSCS");
    _ = ruma_common::event_id!("$39hvsi03hlne:example.com");
    _ = ruma_common::event_id!("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk");
    _ = ruma_common::mxc_uri!("mxc://myserver.fish/sdfdsfsdfsdfgsdfsd");
    _ = ruma_common::room_alias_id!("#alias:server.tld");
    _ = ruma_common::room_id!("!1234567890:matrix.org");
    _ = ruma_common::room_version_id!("1");
    _ = ruma_common::room_version_id!("1-custom");
    _ = ruma_common::server_signing_key_id!("ed25519:Abc_1");
    _ = ruma_common::server_name!("myserver.fish");
    _ = ruma_common::user_id!("@user:ruma.io");

    _ = ruma_common::owned_user_id!("@user:ruma.io");
}
