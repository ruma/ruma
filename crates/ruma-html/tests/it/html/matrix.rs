use assert_matches2::assert_matches;
use ruma_html::{
    matrix::{AnchorUri, MatrixElement},
    Html,
};

#[test]
fn elements() {
    let raw_html = "\
        <h1>Title</h1>\
        <div class=\"text\">\
            <p>This is some <em>text</em></p>\
        </div>\
        <marquee id=\"scrolling_text\">This is scrolling</marquee>
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    // `<h1>` element.
    let h1_node = html_children.next().unwrap();
    let h1_element = h1_node.as_element().unwrap().to_matrix();
    assert_matches!(h1_element.element, MatrixElement::H(heading));
    assert_eq!(heading.level, 1);
    assert!(h1_element.attrs.is_empty());

    // `<div>` element.
    let div_node = html_children.next().unwrap();
    let div_element = div_node.as_element().unwrap().to_matrix();
    assert_matches!(div_element.element, MatrixElement::Div);
    // The `class` attribute is not supported.
    assert_eq!(div_element.attrs.len(), 1);

    // `<p>` element.
    let p_node = div_node.first_child().unwrap();
    let p_element = p_node.as_element().unwrap().to_matrix();
    assert_matches!(p_element.element, MatrixElement::P);
    assert!(p_element.attrs.is_empty());

    // Text of `<p>` element.
    let p_text_node = p_node.first_child().unwrap();

    // `<em>` element.
    let em_node = p_text_node.next_sibling().unwrap();
    let em_element = em_node.as_element().unwrap().to_matrix();
    assert_matches!(em_element.element, MatrixElement::Em);
    assert!(em_element.attrs.is_empty());

    // `<marquee>` element.
    let marquee_node = html_children.next().unwrap();
    let marquee_element = marquee_node.as_element().unwrap().to_matrix();
    assert_matches!(marquee_element.element, MatrixElement::Other(_));
    assert_eq!(marquee_element.attrs.len(), 1);
}

#[test]
fn span_attributes() {
    let raw_html = "\
        <span \
            data-mx-color=\"#00ff00\" \
            data-mx-bg-color=\"#ff0000\" \
            data-mx-spoiler \
            data-mx-spoiler=\"This is a spoiler\"\
        >\
            Hidden and colored\
        </span>\
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    let span_node = html_children.next().unwrap();
    let span_element = span_node.as_element().unwrap().to_matrix();

    assert_matches!(span_element.element, MatrixElement::Span(span));

    assert_eq!(span.color.unwrap().as_ref(), "#00ff00");
    assert_eq!(span.bg_color.unwrap().as_ref(), "#ff0000");
    // Uses the first spoiler attribute, the second is dropped.
    assert!(span.spoiler.unwrap().is_empty());

    assert!(span_element.attrs.is_empty());
}

#[test]
fn a_attributes() {
    let raw_html = "\
        <a \
            name=\"my_anchor\" \
            target=\"_blank\" \
            href=\"https://localhost/\"\
        >\
            Link with all supported attributes\
        </a>\
        <a href=\"matrix:r/somewhere:localhost\">Link with valid matrix scheme URI</a>\
        <a href=\"matrix:somewhere:localhost\">Link with invalid matrix scheme URI</a>\
        <a href=\"https://matrix.to/#/%23somewhere:example.org\">Link with valid matrix.to URI</a>\
        <a href=\"https://matrix.to/#/somewhere:example.org\">Link with invalid matrix.to URI</a>\
        <a href=\"ruma:html\">Link with unsupported scheme</a>\
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    // First `<a>` element, with all supported attributes.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert_eq!(anchor.name.unwrap().as_ref(), "my_anchor");
    assert_eq!(anchor.target.unwrap().as_ref(), "_blank");
    assert_matches!(anchor.href.unwrap(), AnchorUri::Other(uri));
    assert_eq!(uri.as_ref(), "https://localhost/");
    assert!(element.attrs.is_empty());

    // Second `<a>` element, with valid matrix scheme URI.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert!(anchor.name.is_none());
    assert!(anchor.target.is_none());
    assert_matches!(anchor.href.unwrap(), AnchorUri::Matrix(uri));
    assert_eq!(uri.to_string(), "matrix:r/somewhere:localhost");
    assert!(element.attrs.is_empty());

    // Third `<a>` element, with invalid matrix scheme URI.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert!(anchor.name.is_none());
    assert!(anchor.target.is_none());
    assert!(anchor.href.is_none());
    // The `href` attribute is in the unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Fourth `<a>` element, with valid matrix.to URI.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert!(anchor.name.is_none());
    assert!(anchor.target.is_none());
    assert_matches!(anchor.href.unwrap(), AnchorUri::MatrixTo(uri));
    assert_eq!(uri.to_string(), "https://matrix.to/#/%23somewhere:example.org");
    assert!(element.attrs.is_empty());

    // Fifth `<a>` element, with invalid matrix.to URI.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert!(anchor.name.is_none());
    assert!(anchor.target.is_none());
    assert!(anchor.href.is_none());
    // The `href` attribute is in the unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Sixth `<a>` element, with unsupported scheme.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::A(anchor));
    assert!(anchor.name.is_none());
    assert!(anchor.target.is_none());
    assert!(anchor.href.is_none());
    // The `href` attribute is in the unsupported attributes.
    assert_eq!(element.attrs.len(), 1);
}

#[test]
fn img_attributes() {
    let raw_html = "\
        <img \
            width=200 \
            height=200 \
            alt=\"Image with valid attributes\" \
            title=\"Image with valid attributes\" \
            src=\"mxc://localhost/abc123\" \
        />\
        <img \
            width=\"\" \
            height=\"\" \
            alt=\"Image with invalid attributes\" \
            title=\"Image with invalid attributes\" \
            src=\"https://localhost/abc123.png\" \
        />\
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    // First `<img>` element, with valid attributes.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Img(image));
    assert_eq!(image.width.unwrap(), 200);
    assert_eq!(image.height.unwrap(), 200);
    assert_eq!(image.alt.unwrap().as_ref(), "Image with valid attributes");
    assert_eq!(image.title.unwrap().as_ref(), "Image with valid attributes");
    assert_eq!(image.src.unwrap(), "mxc://localhost/abc123");
    assert!(element.attrs.is_empty());

    // Second `<img>` element, with invalid attributes.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Img(image));
    assert!(image.width.is_none());
    assert!(image.height.is_none());
    assert_eq!(image.alt.unwrap().as_ref(), "Image with invalid attributes");
    assert_eq!(image.title.unwrap().as_ref(), "Image with invalid attributes");
    assert!(image.src.is_none());
    // Invalid attributes are in the unsupported attributes.
    assert_eq!(element.attrs.len(), 3);
}

#[test]
fn ol_attributes() {
    let raw_html = "\
        <ol start=2>\
            <li>Item in list with valid start attribute</li>\
        </ol>\
        <ol start=\"beginning\">\
            <li>Item in list with invalid start attribute</li>\
        </ol>\
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    // First `<ol>` element, with valid `start` attribute.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Ol(ol));
    assert_eq!(ol.start.unwrap(), 2);
    assert!(element.attrs.is_empty());

    // First `<ol>` element, with invalid `start` attribute.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Ol(ol));
    assert!(ol.start.is_none());
    assert_eq!(element.attrs.len(), 1);
}

#[test]
fn code_attributes() {
    let raw_html = "\
        <code class=\"language-rust\">\
            let s = \"Code with only `language-` class\";\
        </code>\
        <code class=\"rust-code\">\
            let s = \"Code with other class\";\
        </code>\
        <code class=\"language-rust rust-code\">\
            let s = \"Code with several classes beginning with `language-` class\";\
        </code>\
        <code class=\"rust-code language-rust\">\
            let s = \"Code with several classes not beginning with `language-` class\";\
        </code>\
        <code class=\"language-\">\
            let s = \"Code with invalid `language-` class\";\
        </code>\
        <code class=\"code-language-rust\">\
            let s = \"Code with other class containing `language-`\";\
        </code>\
    ";
    let html = Html::parse(raw_html);
    let mut html_children = html.children();

    // First `<code>` element, with only `language-` class.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert_eq!(code.language.unwrap().as_ref(), "rust");
    assert!(element.attrs.is_empty());

    // Second `<code>` element, with other class.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert!(code.language.is_none());
    // `class` is in unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Third `<code>` element, with several classes beginning with `language-` class.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert_eq!(code.language.unwrap().as_ref(), "rust");
    // Because it contains other classes, `class` is also in unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Fourth `<code>` element, with several classes not beginning with `language-` class.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert_eq!(code.language.unwrap().as_ref(), "rust");
    // Because it contains other classes, `class` is also in unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Fifth `<code>` element, with invalid `language-` class.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert!(code.language.is_none());
    // `class` is in unsupported attributes.
    assert_eq!(element.attrs.len(), 1);

    // Sixth `<code>` element,  with other class containing `language-`.
    let node = html_children.next().unwrap();
    let element = node.as_element().unwrap().to_matrix();

    assert_matches!(element.element, MatrixElement::Code(code));
    assert!(code.language.is_none());
    // `class` is in unsupported attributes.
    assert_eq!(element.attrs.len(), 1);
}
