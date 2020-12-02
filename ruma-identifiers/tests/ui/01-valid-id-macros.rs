fn main() {
    let _ = ruma_identifiers::device_key_id!("ed25519:JLAFKJWSCS");
    let _ = ruma_identifiers::event_id!("$39hvsi03hlne:example.com");
    let _ = ruma_identifiers::event_id!("$acR1l0raoZnm60CBwAVgqbZqoO/mYU81xysh1u7XcJk");
    let _ = ruma_identifiers::room_alias_id!("#alias:server.tld");
    let _ = ruma_identifiers::room_id!("!1234567890:matrix.org");
    let _ = ruma_identifiers::room_version_id!("1");
    let _ = ruma_identifiers::room_version_id!("1-custom");
    let _ = ruma_identifiers::server_signing_key_id!("ed25519:Abc_1");
    let _ = ruma_identifiers::server_name!("myserver.fish");
    let _ = ruma_identifiers::user_id!("@user:ruma.io");
}
