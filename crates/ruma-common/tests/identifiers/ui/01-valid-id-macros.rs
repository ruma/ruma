fn main() {
    let _ = ruma_common::device_key_id!("ed25519:JLAFKJWSCS");
    let _ = ruma_common::event_id!("$39hvsi03hlne:example.com");
    let _ = ruma_common::event_id!("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk");
    let _ = ruma_common::mxc_uri!("mxc://myserver.fish/sdfdsfsdfsdfgsdfsd");
    let _ = ruma_common::room_alias_id!("#alias:server.tld");
    let _ = ruma_common::room_id!("!1234567890:matrix.org");
    let _ = ruma_common::room_version_id!("1");
    let _ = ruma_common::room_version_id!("1-custom");
    let _ = ruma_common::server_signing_key_id!("ed25519:Abc_1");
    let _ = ruma_common::server_name!("myserver.fish");
    let _ = ruma_common::user_id!("@user:ruma.io");
}
