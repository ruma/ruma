use ruma_html::Html;

#[test]
fn navigate_tree() {
    let raw_html = "\
        <h1>Title</h1>\
        <div class=\"text\">\
            <p>This is some <em>text</em></p>\
        </div>\
    ";
    let html = Html::parse(raw_html);

    assert!(html.has_children());
    assert!(html.first_child().is_some());
    assert!(html.last_child().is_some());

    let mut html_children = html.children();

    // `<h1>` element.
    let h1_node = html_children.next().unwrap();

    let h1_element = h1_node.as_element().unwrap();
    assert_eq!(&h1_element.name.local, "h1");
    assert!(h1_element.attrs.is_empty());

    assert!(h1_node.parent().is_none());
    assert!(h1_node.next_sibling().is_some());
    assert!(h1_node.prev_sibling().is_none());
    assert!(h1_node.has_children());
    assert!(h1_node.first_child().is_some());
    assert!(h1_node.last_child().is_some());

    let mut h1_children = h1_node.children();

    // Text of `<h1>` element.
    let h1_text_node = h1_children.next().unwrap();
    let h1_text = h1_text_node.as_text().unwrap();
    assert_eq!(h1_text.as_ref(), "Title");

    assert!(h1_text_node.parent().is_some());
    assert!(h1_text_node.next_sibling().is_none());
    assert!(h1_text_node.prev_sibling().is_none());
    assert!(!h1_text_node.has_children());
    assert!(h1_text_node.first_child().is_none());
    assert!(h1_text_node.last_child().is_none());

    let mut h1_text_children = h1_text_node.children();
    assert!(h1_text_children.next().is_none());

    assert!(h1_children.next().is_none());

    // `<div>` element.
    let div_node = html_children.next().unwrap();

    let div_element = div_node.as_element().unwrap();
    assert_eq!(&div_element.name.local, "div");
    assert_eq!(div_element.attrs.len(), 1);
    let class_attr = div_element.attrs.first().unwrap();
    assert_eq!(&class_attr.name.local, "class");
    assert_eq!(class_attr.value.as_ref(), "text");

    assert!(div_node.parent().is_none());
    assert!(div_node.next_sibling().is_none());
    assert!(div_node.prev_sibling().is_some());
    assert!(div_node.has_children());
    assert!(div_node.first_child().is_some());
    assert!(div_node.last_child().is_some());

    let mut div_children = div_node.children();

    // `<p>` element.
    let p_node = div_children.next().unwrap();

    let p_element = p_node.as_element().unwrap();
    assert_eq!(&p_element.name.local, "p");
    assert!(p_element.attrs.is_empty());

    assert!(p_node.parent().is_some());
    assert!(p_node.next_sibling().is_none());
    assert!(p_node.prev_sibling().is_none());
    assert!(p_node.has_children());
    assert!(p_node.first_child().is_some());
    assert!(p_node.last_child().is_some());

    let mut p_children = p_node.children();

    // Text of `<p>` element.
    let p_text_node = p_children.next().unwrap();
    let p_text = p_text_node.as_text().unwrap();
    assert_eq!(p_text.as_ref(), "This is some ");

    assert!(p_text_node.parent().is_some());
    assert!(p_text_node.next_sibling().is_some());
    assert!(p_text_node.prev_sibling().is_none());
    assert!(!p_text_node.has_children());
    assert!(p_text_node.first_child().is_none());
    assert!(p_text_node.last_child().is_none());

    let mut p_text_children = p_text_node.children();
    assert!(p_text_children.next().is_none());

    // `<em>` element.
    let em_node = p_children.next().unwrap();

    let em_element = em_node.as_element().unwrap();
    assert_eq!(&em_element.name.local, "em");
    assert!(em_element.attrs.is_empty());

    assert!(em_node.parent().is_some());
    assert!(em_node.next_sibling().is_none());
    assert!(em_node.prev_sibling().is_some());
    assert!(em_node.has_children());
    assert!(em_node.first_child().is_some());
    assert!(em_node.last_child().is_some());

    let mut em_children = em_node.children();

    // Text of `<em>` element.
    let em_text_node = em_children.next().unwrap();
    let em_text = em_text_node.as_text().unwrap();
    assert_eq!(em_text.as_ref(), "text");

    assert!(em_text_node.parent().is_some());
    assert!(em_text_node.next_sibling().is_none());
    assert!(em_text_node.prev_sibling().is_none());
    assert!(!em_text_node.has_children());
    assert!(em_text_node.first_child().is_none());
    assert!(em_text_node.last_child().is_none());

    let mut em_text_children = em_text_node.children();
    assert!(em_text_children.next().is_none());

    assert!(em_children.next().is_none());

    assert!(p_children.next().is_none());

    assert!(div_children.next().is_none());

    assert!(html_children.next().is_none());
}
