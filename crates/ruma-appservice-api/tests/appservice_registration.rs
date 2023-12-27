use assert_matches2::assert_matches;
use ruma_appservice_api::Registration;

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
    let observed: Registration = serde_yaml::from_str(registration_config).unwrap();

    assert_eq!(observed.id, "IRC Bridge");
    assert_eq!(observed.url.unwrap(), "http://127.0.0.1:1234");
    assert_eq!(
        observed.as_token,
        "30c05ae90a248a4188e620216fa72e349803310ec83e2a77b34fe90be6081f46"
    );
    assert_eq!(
        observed.hs_token,
        "312df522183efd404ec1cd22d2ffa4bbc76a8c1ccf541dd692eef281356bb74e"
    );
    assert_eq!(observed.sender_localpart, "_irc_bot");
    assert_eq!(observed.rate_limited, None);
    assert_eq!(observed.protocols, None);

    assert_eq!(observed.namespaces.users.len(), 1);
    assert!(observed.namespaces.users[0].exclusive);
    assert_eq!(observed.namespaces.users[0].regex, "@_irc_bridge_.*");

    assert_eq!(observed.namespaces.aliases.len(), 1);
    assert!(!observed.namespaces.aliases[0].exclusive);
    assert_eq!(observed.namespaces.aliases[0].regex, "#_irc_bridge_.*");

    assert_eq!(observed.namespaces.rooms.len(), 0);
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
    assert_matches!(serde_yaml::from_str(registration_config).unwrap(), Registration { url, .. });
    assert_eq!(url, None);
}
