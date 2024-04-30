use ruma_html::{
    ElementAttributesReplacement, ElementAttributesSchemes, Html, ListBehavior, NameReplacement,
    PropertiesNames, SanitizerConfig,
};

#[test]
fn strict_mode_valid_input() {
    let config = SanitizerConfig::strict().remove_reply_fallback();
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

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
fn strict_mode_elements_remove() {
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
    html.sanitize_with(&config);

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
fn strict_mode_elements_reply_remove() {
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
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        This has no tag\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn remove_only_reply_fallback() {
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
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <keep-me>This keeps its tag</keep-me>\
        <p>But this is inside a tag</p>\
        "
    );
}

#[test]
fn strict_mode_attrs_remove() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <h1 id=\"anchor1\">Title for important stuff</h1>\
        <p class=\"important\">Look at <span data-mx-color=\"#0000ff\" size=20>me!</span></p>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <h1>Title for important stuff</h1>\
        <p>Look at <span data-mx-color=\"#0000ff\">me!</span></p>\
        "
    );
}

#[test]
fn strict_mode_img_remove_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Look at that picture:</p>\
        <img src=\"https://notareal.hs/abcdef\">\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(html.to_string(), "<p>Look at that picture:</p>");
}

#[test]
fn strict_mode_link_remove_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Go see <a href=\"file://local/file.html\">my local website</a></p>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <p>Go see my local website</p>\
        "
    );
}

#[test]
fn compat_mode_link_remove_scheme() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        ",
    );
    html.sanitize_with(&config);
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
    html.sanitize_with(&config);
    assert_eq!(
        html.to_string(),
        "\
        <p>Join <a href=\"matrix:r/myroom:notareal.hs\">my room</a></p>\
        <p>To talk about <a href=\"https://mycat.org\">my cat</a></p>\
        "
    );
}

#[test]
fn strict_mode_class_remove() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <pre><code class=\"language-rust custom-class\">
            type StringList = Vec&lt;String&gt;;
        </code></pre>\
        <p>What do you think of the name <code class=\"fake-language-rust\">StringList</code>?</p>\
        ",
    );
    html.sanitize_with(&config);

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
fn strict_mode_depth_remove() {
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
    html.sanitize_with(&config);

    let res = html.to_string();
    assert!(res.contains("I should be fine."));
    assert!(!res.contains("I am in too deep!"));
}

#[test]
fn strict_mode_replace_deprecated() {
    let config = SanitizerConfig::strict();
    let mut html = Html::parse(
        "\
        <p>Look at <strike>you </strike><font data-mx-bg-color=\"#ff0000\" color=\"#0000ff\">me!</span></p>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <p>Look at <s>you </s><span data-mx-bg-color=\"#ff0000\" data-mx-color=\"#0000ff\">me!</span></p>\
        "
    );
}

#[test]
fn allow_elements() {
    let config = SanitizerConfig::new().allow_elements(["ul", "li", "p", "img"], ListBehavior::Add);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph with some color</p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        &lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;\
        "
    );
}

#[test]
fn override_elements() {
    let config =
        SanitizerConfig::strict().allow_elements(["ul", "li", "p", "img"], ListBehavior::Override);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph with some color</p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        &lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;\
        "
    );
}

#[test]
fn add_elements() {
    let config = SanitizerConfig::strict().allow_elements(["keep-me"], ListBehavior::Add);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <keep-me>I was kept!</keep-me>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <keep-me>I was kept!</keep-me>\
        "
    );
}

#[test]
fn remove_elements() {
    let config = SanitizerConfig::strict().remove_elements(["span", "code"]);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph </p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        "
    );
}

#[test]
fn ignore_elements() {
    let config = SanitizerConfig::new().ignore_elements(["span", "code"]);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph with some color</p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        &lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;\
        "
    );
}

#[test]
fn replace_elements() {
    let config = SanitizerConfig::new()
        .replace_elements([NameReplacement { old: "ul", new: "ol" }], ListBehavior::Add);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ol><li>This</li><li>has</li><li>no</li><li>tag</li></ol>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn replace_elements_override() {
    let config = SanitizerConfig::strict()
        .replace_elements([NameReplacement { old: "ul", new: "ol" }], ListBehavior::Override);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <strike>This is wrong</strike>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ol><li>This</li><li>has</li><li>no</li><li>tag</li></ol>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        This is wrong\
        "
    );
}

#[test]
fn replace_elements_add() {
    let config = SanitizerConfig::strict()
        .replace_elements([NameReplacement { old: "ul", new: "ol" }], ListBehavior::Add);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <strike>This is wrong</strike>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ol><li>This</li><li>has</li><li>no</li><li>tag</li></ol>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <s>This is wrong</s>\
        "
    );
}

#[test]
fn allow_attributes() {
    let config = SanitizerConfig::new().allow_attributes(
        [PropertiesNames { parent: "img", properties: &["src"] }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span>with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code>&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn override_attributes() {
    let config = SanitizerConfig::strict().allow_attributes(
        [PropertiesNames { parent: "img", properties: &["src"] }],
        ListBehavior::Override,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span>with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code>&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn add_attributes() {
    let config = SanitizerConfig::strict().allow_attributes(
        [PropertiesNames { parent: "img", properties: &["id"] }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img id=\"my_image\" src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img id=\"my_image\" src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn remove_attributes() {
    let config = SanitizerConfig::strict()
        .remove_attributes([PropertiesNames { parent: "span", properties: &["data-mx-color"] }]);
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span>with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn replace_attributes() {
    let config = SanitizerConfig::new().replace_attributes(
        [ElementAttributesReplacement {
            element: "span",
            replacements: &[NameReplacement { old: "data-mx-color", new: "data-mx-bg-color" }],
        }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-bg-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn replace_attributes_override() {
    let config = SanitizerConfig::strict().replace_attributes(
        [ElementAttributesReplacement {
            element: "font",
            replacements: &[NameReplacement { old: "color", new: "data-mx-bg-color" }],
        }],
        ListBehavior::Override,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <font color=\"green\">with some color</font></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-bg-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn replace_attributes_add() {
    let config = SanitizerConfig::strict().replace_attributes(
        [ElementAttributesReplacement {
            element: "img",
            replacements: &[NameReplacement { old: "alt", new: "title" }],
        }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <font color=\"green\">with some color</font></p>\
        <img alt=\"An image\" src=\"mxc://notareal.hs/abcdef\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <ul><li>This</li><li>has</li><li>no</li><li>tag</li></ul>\
        <p>This is a paragraph <span data-mx-color=\"green\">with some color</span></p>\
        <img src=\"mxc://notareal.hs/abcdef\" title=\"An image\">\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn allow_schemes() {
    let config = SanitizerConfig::new().allow_schemes(
        [ElementAttributesSchemes {
            element: "img",
            attr_schemes: &[PropertiesNames { parent: "src", properties: &["mxc"] }],
        }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <img src=\"https://notareal.hs/abcdef.png\">\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <img src=\"mxc://notareal.hs/abcdef\">\
        "
    );
}

#[test]
fn override_schemes() {
    let config = SanitizerConfig::strict().allow_schemes(
        [ElementAttributesSchemes {
            element: "img",
            attr_schemes: &[PropertiesNames { parent: "src", properties: &["https"] }],
        }],
        ListBehavior::Override,
    );
    let mut html = Html::parse(
        "\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <img src=\"https://notareal.hs/abcdef.png\">\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <img src=\"https://notareal.hs/abcdef.png\">\
        "
    );
}

#[test]
fn add_schemes() {
    let config = SanitizerConfig::strict().allow_schemes(
        [ElementAttributesSchemes {
            element: "img",
            attr_schemes: &[PropertiesNames { parent: "src", properties: &["https"] }],
        }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <img src=\"https://notareal.hs/abcdef.png\">\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <img src=\"mxc://notareal.hs/abcdef\">\
        <img src=\"https://notareal.hs/abcdef.png\">\
        "
    );
}

#[test]
fn deny_schemes() {
    let config = SanitizerConfig::strict().deny_schemes([ElementAttributesSchemes {
        element: "a",
        attr_schemes: &[PropertiesNames { parent: "href", properties: &["http"] }],
    }]);
    let mut html = Html::parse(
        "\
        <a href=\"https://notareal.hs/abcdef.png\">Secure link to an image</a>\
        <a href=\"http://notareal.hs/abcdef.png\">Insecure link to an image</a>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <a href=\"https://notareal.hs/abcdef.png\">Secure link to an image</a>\
        Insecure link to an image\
        "
    );
}

#[test]
fn allow_classes() {
    let config = SanitizerConfig::new().allow_classes(
        [PropertiesNames { parent: "img", properties: &["custom-class", "custom-class-*"] }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <img class=\"custom-class custom-class-img img\" src=\"mxc://notareal.hs/abcdef\">\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <code>&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        <img class=\"custom-class custom-class-img\" src=\"mxc://notareal.hs/abcdef\">\
        "
    );
}

#[test]
fn override_classes() {
    let config = SanitizerConfig::strict().allow_classes(
        [PropertiesNames { parent: "code", properties: &["custom-class", "custom-class-*"] }],
        ListBehavior::Override,
    );
    let mut html = Html::parse(
        "\
        <code class=\"language-html custom-class custom-class-code code\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <code class=\"custom-class custom-class-code\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn add_classes() {
    let config = SanitizerConfig::strict().allow_classes(
        [PropertiesNames { parent: "code", properties: &["custom-class", "custom-class-*"] }],
        ListBehavior::Add,
    );
    let mut html = Html::parse(
        "\
        <code class=\"language-html custom-class custom-class-code code\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <code class=\"language-html custom-class custom-class-code\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}

#[test]
fn remove_classes() {
    let config = SanitizerConfig::strict()
        .remove_classes([PropertiesNames { parent: "code", properties: &["language-rust"] }]);
    let mut html = Html::parse(
        "\
        <code class=\"language-html language-rust\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        ",
    );
    html.sanitize_with(&config);

    assert_eq!(
        html.to_string(),
        "\
        <code class=\"language-html\">&lt;mx-reply&gt;This is a fake reply&lt;/mx-reply&gt;</code>\
        "
    );
}
