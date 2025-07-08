use ruma_html::{
    HtmlSanitizerMode, RemoveReplyFallback, remove_html_reply_fallback, sanitize_html,
};

#[test]
fn sanitize() {
    let sanitized = sanitize_html(
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                <br>\
                Previous message\
            </blockquote>\
        </mx-reply>\
        <removed>This has no tag</removed>\
        <p>But this is inside a tag</p>\
        ",
        HtmlSanitizerMode::Strict,
        RemoveReplyFallback::No,
    );

    assert_eq!(
        sanitized,
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                <br>\
                Previous message\
            </blockquote>\
        </mx-reply>\
        This has no tag\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn sanitize_without_reply() {
    let sanitized = sanitize_html(
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                <br>\
                Previous message\
            </blockquote>\
        </mx-reply>\
        <removed>This has no tag</removed>\
        <p>But this is inside a tag</p>\
        ",
        HtmlSanitizerMode::Strict,
        RemoveReplyFallback::Yes,
    );

    assert_eq!(
        sanitized,
        "\
        This has no tag\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn remove_html_reply() {
    let without_reply = remove_html_reply_fallback(
        "\
        <mx-reply>\
            <blockquote>\
                <a href=\"https://matrix.to/#/!n8f893n9:example.com/$1598361704261elfgc:localhost\">In reply to</a> \
                <a href=\"https://matrix.to/#/@alice:example.com\">@alice:example.com</a>\
                <br>\
                Previous message\
            </blockquote>\
        </mx-reply>\
        <keep-me>This keeps its tag</keep-me>\
        <p>But this is inside a tag</p>\
        ",
    );

    assert_eq!(
        without_reply,
        "\
        <keep-me>This keeps its tag</keep-me>\
        <p>But this is inside a tag</p>\
        "
    );
}
