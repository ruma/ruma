use matches::assert_matches;
use ruma_appservice_api::{Namespace, Namespaces, Registration};

#[test]
fn registration_deserialization() {
    let registration_config = r##"
        id: "IRC Bridge"
        url: "http://127.0.0.1:1234"
        as_token: "30c05ae90a248a4188e620216fa72e349803310ec83e2a77b34fe90be6081f46"
        hs_token: "312df522183efd404ec1cd22d2ffa4bbc76a8c1ccf541dd692eef281356bb74e"
        sender_localpart: "_irc_bot"
        namespaces:
          users:
            - exclusive: true
              regex: "@_irc_bridge_.*"
          aliases:
            - exclusive: false
              regex: "#_irc_bridge_.*"
          rooms: []
        "##;
    let observed = serde_yaml::from_str(&registration_config).unwrap();
    assert_matches!(
        observed,
        Registration {
            id,
            url,
            as_token,
            hs_token,
            sender_localpart,
            rate_limited,
            protocols,
            namespaces: Namespaces { users, aliases, rooms, .. },
            ..
        }
        if id == "IRC Bridge"
            && url == "http://127.0.0.1:1234"
            && as_token == "30c05ae90a248a4188e620216fa72e349803310ec83e2a77b34fe90be6081f46"
            && hs_token == "312df522183efd404ec1cd22d2ffa4bbc76a8c1ccf541dd692eef281356bb74e"
            && sender_localpart == "_irc_bot"
            && rate_limited == None
            && protocols == None
            && users[0] == Namespace::new(true, "@_irc_bridge_.*".into())
            && aliases[0] == Namespace::new(false, "#_irc_bridge_.*".into())
            && rooms.is_empty()
    );
}

#[test]
fn config_with_optional_url() {
    let registration_config = r#"
        id: "IRC Bridge"
        url: null
        as_token: "30c05ae90a248a4188e620216fa72e349803310ec83e2a77b34fe90be6081f46"
        hs_token: "312df522183efd404ec1cd22d2ffa4bbc76a8c1ccf541dd692eef281356bb74e"
        sender_localpart: "_irc_bot"
        namespaces:
          users: []
          aliases: []
          rooms: []
        "#;
    assert_matches!(
        serde_yaml::from_str(&registration_config).unwrap(),
        Registration { url, .. } if url == "null"
    );
}
