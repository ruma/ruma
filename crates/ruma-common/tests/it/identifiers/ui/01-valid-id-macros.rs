fn main() {
    _ = ruma_common::base64_public_key!("self+signing+master+public+key");
    _ = ruma_common::device_id!("MYDEVICE");
    _ = ruma_common::event_id!("$39hvsi03hlne:example.com");
    _ = ruma_common::event_id!("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk");
    _ = ruma_common::mxc_uri!("mxc://myserver.fish/sdfdsfsdfsdfgsdfsd");
    _ = ruma_common::room_alias_id!("#alias:server.tld");
    _ = ruma_common::room_id!("!1234567890:matrix.org");
    _ = ruma_common::room_version_id!("1");
    _ = ruma_common::room_version_id!("1-custom");
    _ = ruma_common::server_name!("myserver.fish");
    _ = ruma_common::server_signing_key_version!("Abc_1");
    _ = ruma_common::user_id!("@user:ruma.io");

    _ = ruma_common::owned_user_id!("@user:ruma.io");

    #[cfg(feature = "unstable-identifier-ref-macros")]
    {
        _ = ruma_common::base64_public_key_ref!("self+signing+master+public+key");
        _ = ruma_common::device_id_ref!("MYDEVICE");
        _ = ruma_common::event_id_ref!("$39hvsi03hlne:example.com");
        _ = ruma_common::event_id_ref!("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk");
        _ = ruma_common::mxc_uri_ref!("mxc://myserver.fish/sdfdsfsdfsdfgsdfsd");
        _ = ruma_common::room_alias_id_ref!("#alias:server.tld");
        _ = ruma_common::room_id_ref!("!1234567890:matrix.org");
        _ = ruma_common::server_name_ref!("myserver.fish");
        _ = ruma_common::server_signing_key_version_ref!("Abc_1");
    }
}
