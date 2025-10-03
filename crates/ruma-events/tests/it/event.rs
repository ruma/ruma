use ruma_events::GlobalAccountDataEventContent;
use ruma_macros::{Event, EventContent};
use serde::{Deserialize, Serialize};
use serde_json::{from_value as from_json_value, json};

const TAG: &str = "you're it!";

#[derive(Clone, Debug, Deserialize, Serialize, EventContent)]
#[ruma_event(type = "m.macro.test", kind = GlobalAccountData)]
struct MacroTestContent {
    tag: String,
}

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    // rustc overflows when compiling this see:
    // https://github.com/rust-lang/rust/issues/55779
    // there is a workaround in the file.
    t.pass("tests/it/ui/04-event-sanity-check.rs");
    t.compile_fail("tests/it/ui/05-named-fields.rs");
    t.compile_fail("tests/it/ui/06-no-content-field.rs");
}

#[test]
fn default_attribute() {
    let json_with_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "flag": true,
    });
    let json_without_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
    });

    // Event that requires the flag.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag.clone())
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_without_flag.clone())
            .unwrap_err();
    }

    // Event that doesn't require the flag.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            #[ruma_event(default)]
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag).unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_without_flag).unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(!event.flag);
    }
}

#[test]
fn rename_attribute() {
    let json_with_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "flag": true,
    });
    let json_with_unstable_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "unstable_flag": true,
    });

    // Event with field not renamed.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag.clone())
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(
            json_with_unstable_flag.clone(),
        )
        .unwrap_err();
    }

    // Event with field renamed.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            #[ruma_event(rename = "unstable_flag")]
            flag: bool,
        }

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag).unwrap_err();

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_unstable_flag)
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);
    }
}

#[test]
fn alias_attribute() {
    let json_with_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "flag": true,
    });
    let json_with_unstable_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "unstable_flag": true,
    });
    let json_with_alt_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "alt_flag": true,
    });

    // Event with field not renamed and no alias.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag.clone())
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(
            json_with_unstable_flag.clone(),
        )
        .unwrap_err();

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_alt_flag.clone())
            .unwrap_err();
    }

    // Event with field not renamed and aliases.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            #[ruma_event(alias = "unstable_flag", alias = "alt_flag")]
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag.clone())
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        let event = from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(
            json_with_unstable_flag.clone(),
        )
        .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_alt_flag.clone())
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);
    }

    // Event with field renamed and alias.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            #[ruma_event(rename = "unstable_flag", alias = "alt_flag")]
            flag: bool,
        }

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_flag).unwrap_err();

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_unstable_flag)
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        let event = from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_alt_flag)
            .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);
    }
}

#[test]
fn default_on_error_attribute() {
    let json_with_boolean_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "flag": true,
    });
    let json_with_string_flag = json!({
        "content": {
            "tag": TAG,
        },
        "type": "m.macro.test",
        "flag": "true",
    });

    // Event with propagated error.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            flag: bool,
        }

        let event = from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(
            json_with_boolean_flag.clone(),
        )
        .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_string_flag.clone())
            .unwrap_err();
    }

    // Event with ignored error.
    {
        #[derive(Clone, Debug, Event)]
        struct GlobalAccountDataEvent<C: GlobalAccountDataEventContent> {
            content: C,
            #[ruma_event(default_on_error)]
            flag: bool,
        }

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_boolean_flag)
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(event.flag);

        let event =
            from_json_value::<GlobalAccountDataEvent<MacroTestContent>>(json_with_string_flag)
                .unwrap();
        assert_eq!(event.content.tag, TAG);
        assert!(!event.flag);
    }
}
