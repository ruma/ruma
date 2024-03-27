use ruma_html::{Html, SanitizerConfig};

#[test]
fn valid_input() {
    let config = SanitizerConfig::strict().remove_reply_fallback();
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn tags_remove() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
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
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
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
fn tags_remove_without_reply() {
    let config = SanitizerConfig::strict().remove_reply_fallback();
    let mut html = Html::parse(
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
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        This has no tag\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn tags_remove_only_reply_fallback() {
    let config = SanitizerConfig::new().remove_reply_fallback();
    let mut html = Html::parse(
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
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <keep-me>This keeps its tag</keep-me>\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn attrs_remove() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <h1 id=\"anchor1\">Title for important stuff</h1>\
        <p class=\"important\">Look at <span data-mx-color=\"#0000ff\" size=20>me!</span></p>\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <h1>Title for important stuff</h1>\
        <p>Look at <span data-mx-color=\"#0000ff\">me!</span></p>\
        "
    );
}

#[test]
fn img_remove_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Look at that picture:</p>\
        <img src=\"https://notareal.hs/abcdef\">\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(html.to_string(), "<p>Look at that picture:</p>");
}

#[test]
fn link_remove_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Go see <a href=\"file://local/file.html\">my local website</a></p>\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <p>Go see my local website</p>\
        "
    );
}

#[test]
fn link_compat_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        ",
    );
    html.sanitize_with(config);
    assert_eq!(
        html.to_string(),
        "\
        <p>Join my room</p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        "
    );

    let config = SanitizerConfig::compat();
    let mut html = Html::parse(
        "\
        <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        ",
    );
    html.sanitize_with(config);
    assert_eq!(
        html.to_string(),
        "\
        <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        "
    );
}

#[test]
fn class_remove() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <pre><code class=\"language-rust custom-class\">
            type StringList = Vec&lt;String&gt;;
        </code></pre>\
        <p>What do you think of the name <code class=\"fake-language-rust\">StringList</code>?</p>\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <pre><code class=\"language-rust\">
            type StringList = Vec&lt;String&gt;;
        </code></pre>\
        <p>What do you think of the name <code>StringList</code>?</p>\
        "
    );
}

#[test]
fn depth_remove() {
    let config = SanitizerConfig::strict();
    let deeply_nested_html: String = std::iter::repeat("<div>")
        .take(100)
        .chain(Some(
            "<span>I am in too deep!</span>\
             I should be fine.",
        ))
        .chain(std::iter::repeat("</div>").take(100))
        .collect();

    let mut html = Html::parse(&deeply_nested_html);
    html.sanitize_with(config);

    let res = html.to_string();
    assert!(res.contains("I should be fine."));
    assert!(!res.contains("I am in too deep!"));
}

#[test]
fn replace_deprecated() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Look at <strike>you </strike><font data-mx-bg-color=\"#ff0000\" color=\"#0000ff\">me!</span></p>\
        ",
    );
    html.sanitize_with(config);

    assert_eq!(
        html.to_string(),
        "\
        <p>Look at <s>you </s><span data-mx-bg-color=\"#ff0000\" data-mx-color=\"#0000ff\">me!</span></p>\
        "
    );
}
