use std::{
    cell::RefCell,
    collections::BTreeSet,
    fmt, io,
    iter::FusedIterator,
    rc::{Rc, Weak},
};

use as_variant::as_variant;
use html5ever::{
    local_name, namespace_url, ns, parse_fragment,
    serialize::{serialize, Serialize, SerializeOpts, Serializer, TraversalScope},
    tendril::{StrTendril, TendrilSink},
    tree_builder::{NodeOrText, TreeSink},
    Attribute, LocalName, ParseOpts, QualName,
};
use tracing::debug;

#[cfg(feature = "matrix")]
pub mod matrix;

use crate::SanitizerConfig;

/// An HTML fragment.
///
/// To get the serialized HTML, use its `Display` implementation. Due to the fact that the HTML is
/// parsed, note that malformed HTML and comments will be stripped from the output.
#[derive(Debug)]
pub struct Html {
    document: NodeRef,
}

impl Html {
    /// Construct a new `Html` by parsing the given string.
    ///
    /// This is infallible, any error encountered while parsing the HTML is logged with
    /// `tracing::debug!`.
    pub fn parse(string: &str) -> Self {
        let sink = Self::default();
        let mut parser = parse_fragment(
            sink,
            ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("div")),
            Vec::new(),
        );
        parser.process(string.into());
        parser.finish()
    }

    /// Sanitize this HTML according to the Matrix specification.
    ///
    /// This is equivalent to calling [`Self::sanitize_with()`] with a `config` value of
    /// `SanitizerConfig::compat().remove_reply_fallback()`.
    pub fn sanitize(&self) {
        let config = SanitizerConfig::compat().remove_reply_fallback();
        self.sanitize_with(&config);
    }

    /// Sanitize this HTML according to the given configuration.
    pub fn sanitize_with(&self, config: &SanitizerConfig) {
        config.clean(self);
    }

    /// Get the root node of the HTML.
    fn root(&self) -> NodeRef {
        self.document.first_child().expect("html should always have a root node")
    }

    /// Whether the root node of the HTML has children.
    pub fn has_children(&self) -> bool {
        self.root().has_children()
    }

    /// The first child node of the root node of the HTML.
    ///
    /// Returns `None` if the root node has no children.
    pub fn first_child(&self) -> Option<NodeRef> {
        self.root().first_child()
    }

    /// The last child node of the root node of the HTML .
    ///
    /// Returns `None` if the root node has no children.
    pub fn last_child(&self) -> Option<NodeRef> {
        self.root().last_child()
    }

    /// Iterate through the children of the root node of the HTML.
    pub fn children(&self) -> Children {
        Children::new(self.first_child())
    }
}

impl Default for Html {
    fn default() -> Self {
        Self { document: NodeRef::new(NodeData::Document) }
    }
}

impl TreeSink for Html {
    type Handle = NodeRef;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&self, msg: std::borrow::Cow<'static, str>) {
        debug!("HTML parse error: {msg}");
    }

    fn get_document(&self) -> Self::Handle {
        self.document.clone()
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> html5ever::ExpandedName<'a> {
        target.as_element().expect("not an element").name.expanded()
    }

    fn create_element(
        &self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: html5ever::tree_builder::ElementFlags,
    ) -> Self::Handle {
        NodeRef::new(NodeData::Element(ElementData {
            name,
            attrs: RefCell::new(attrs.into_iter().collect()),
        }))
    }

    fn create_comment(&self, _text: StrTendril) -> Self::Handle {
        NodeRef::new(NodeData::Other)
    }

    fn create_pi(&self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        NodeRef::new(NodeData::Other)
    }

    fn append(&self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(node) => parent.append_child(node),
            NodeOrText::AppendText(text) => {
                // If the previous sibling is also text, add this text to it.
                if let Some(prev_text) =
                    parent.last_child().as_ref().and_then(|sibling| sibling.as_text())
                {
                    prev_text.borrow_mut().push_tendril(&text);
                } else {
                    let node = NodeRef::new(NodeData::Text(text.into()));
                    parent.append_child(node);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if element.0.parent.borrow().is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
    }

    fn get_template_contents(&self, target: &Self::Handle) -> Self::Handle {
        target.clone()
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        Rc::ptr_eq(&x.0, &y.0)
    }

    fn set_quirks_mode(&self, _mode: html5ever::tree_builder::QuirksMode) {}

    fn append_before_sibling(&self, sibling: &Self::Handle, new_node: NodeOrText<Self::Handle>) {
        match new_node {
            NodeOrText::AppendNode(node) => node.insert_before_sibling(sibling),
            NodeOrText::AppendText(text) => {
                // If the previous sibling is also text, add this text to it.
                if let Some(prev_text) =
                    sibling.prev_sibling().as_ref().and_then(|prev_sibling| prev_sibling.as_text())
                {
                    prev_text.borrow_mut().push_tendril(&text);
                } else {
                    let node = NodeRef::new(NodeData::Text(text.into()));
                    node.insert_before_sibling(sibling);
                }
            }
        }
    }

    fn add_attrs_if_missing(&self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let element = target.as_element().unwrap();
        element.attrs.borrow_mut().extend(attrs);
    }

    fn remove_from_parent(&self, target: &Self::Handle) {
        target.detach();
    }

    fn reparent_children(&self, node: &Self::Handle, new_parent: &Self::Handle) {
        for child in node.0.children.take() {
            child.0.parent.take();
            new_parent.append_child(child);
        }
    }
}

impl Serialize for Html {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        match traversal_scope {
            TraversalScope::IncludeNode => {
                for child in self.children() {
                    child.serialize(serializer)?;
                }

                Ok(())
            }
            TraversalScope::ChildrenOnly(_) => Ok(()),
        }
    }
}

impl fmt::Display for Html {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut u8_vec = Vec::new();
        serialize(
            &mut u8_vec,
            self,
            SerializeOpts { traversal_scope: TraversalScope::IncludeNode, ..Default::default() },
        )
        .unwrap();

        f.write_str(&String::from_utf8(u8_vec).unwrap())?;

        Ok(())
    }
}

/// An HTML node.
#[derive(Debug)]
#[non_exhaustive]
struct Node {
    parent: RefCell<Option<Weak<Node>>>,
    children: RefCell<Vec<NodeRef>>,
    data: NodeData,
}

impl Node {
    /// Constructs a new `NodeRef` with the given data.
    fn new(data: NodeData) -> Self {
        Self { parent: Default::default(), children: Default::default(), data }
    }

    /// Returns the data of this `Node` if it is an Element (aka an HTML tag).
    fn as_element(&self) -> Option<&ElementData> {
        as_variant!(&self.data, NodeData::Element)
    }

    /// Returns the text content of this `Node`, if it is a `NodeData::Text`.
    fn as_text(&self) -> Option<&RefCell<StrTendril>> {
        as_variant!(&self.data, NodeData::Text)
    }

    /// Whether this is the root node of the HTML document.
    fn is_root(&self) -> bool {
        // The root node is the `html` element.
        matches!(&self.data, NodeData::Element(element_data) if element_data.name.local.as_bytes() == b"html")
    }

    /// The parent of this node, if any.
    fn parent(&self) -> Option<NodeRef> {
        self.parent.borrow().as_ref()?.upgrade().map(NodeRef)
    }
}

/// The data of a `Node`.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum NodeData {
    /// The root node of the `Html`.
    Document,

    /// A text node.
    Text(RefCell<StrTendril>),

    /// An HTML element (aka a tag).
    Element(ElementData),

    /// Other types (comment, processing instruction, …).
    Other,
}

/// The data of an HTML element.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_structs)]
pub struct ElementData {
    /// The qualified name of the element.
    pub name: QualName,

    /// The attributes of the element.
    pub attrs: RefCell<BTreeSet<Attribute>>,
}

impl ElementData {
    /// Convert this element data to typed data as [suggested by the Matrix Specification][spec].
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    #[cfg(feature = "matrix")]
    pub fn to_matrix(&self) -> matrix::MatrixElementData {
        matrix::MatrixElementData::parse(&self.name, &self.attrs.borrow())
    }
}

/// A reference to an HTML node.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct NodeRef(Rc<Node>);

impl NodeRef {
    /// Constructs a new `NodeRef` with the given data.
    fn new(data: NodeData) -> Self {
        Self(Node::new(data).into())
    }

    /// Detach this node from the tree, if it has a parent.
    pub(crate) fn detach(&self) {
        if let Some((parent, index)) = self.parent_and_index() {
            parent.0.children.borrow_mut().remove(index);
            self.0.parent.take();
        }
    }

    /// Append the given child node to this node.
    ///
    /// The child node is detached from its previous position.
    fn append_child(&self, child: NodeRef) {
        child.detach();

        child.0.parent.replace(Some(Rc::downgrade(&self.0)));
        self.0.children.borrow_mut().push(child);
    }

    /// If this node has a parent, get it and the node's position in the parent's children.
    fn parent_and_index(&self) -> Option<(NodeRef, usize)> {
        let parent = self.0.parent()?;
        let i = parent
            .0
            .children
            .borrow()
            .iter()
            .position(|child| Rc::ptr_eq(&child.0, &self.0))
            .expect("child should be in parent's children");
        Some((parent, i))
    }

    /// Insert this node before the given sibling.
    ///
    /// This node is detached from its previous position.
    pub(crate) fn insert_before_sibling(&self, sibling: &NodeRef) {
        self.detach();

        let (parent, index) = sibling.parent_and_index().expect("sibling should have parent");

        self.0.parent.replace(Some(Rc::downgrade(&parent.0)));
        parent.0.children.borrow_mut().insert(index, self.clone());
    }

    /// Constructs a new element `NodeRef` with the same data as this one, but with a different
    /// element name and use it to replace this one in the parent.
    ///
    /// Panics if this node is not in the tree and is not an element node.
    pub(crate) fn replace_with_element_name(self, name: LocalName) -> NodeRef {
        let mut element_data = self.as_element().unwrap().clone();
        element_data.name.local = name;

        let new_node = NodeRef::new(NodeData::Element(element_data));

        for child in self.children() {
            new_node.append_child(child);
        }

        new_node.insert_before_sibling(&self);
        self.detach();

        new_node
    }

    /// The data of the node.
    pub fn data(&self) -> &NodeData {
        &self.0.data
    }

    /// Returns the data of this `Node` if it is an Element (aka an HTML tag).
    pub fn as_element(&self) -> Option<&ElementData> {
        self.0.as_element()
    }

    /// Returns the text content of this `Node`, if it is a `NodeData::Text`.
    pub fn as_text(&self) -> Option<&RefCell<StrTendril>> {
        self.0.as_text()
    }

    /// The parent node of this node.
    ///
    /// Returns `None` if the parent is the root node.
    pub fn parent(&self) -> Option<NodeRef> {
        let parent = self.0.parent()?;

        // We don't want users to be able to navigate to the root.
        if parent.0.is_root() {
            return None;
        }

        Some(parent)
    }

    /// The next sibling node of this node.
    ///
    /// Returns `None` if this is the last of its siblings.
    pub fn next_sibling(&self) -> Option<NodeRef> {
        let (parent, index) = self.parent_and_index()?;
        let index = index.checked_add(1)?;
        let sibling = parent.0.children.borrow().get(index).cloned();
        sibling
    }

    /// The previous sibling node of this node.
    ///
    /// Returns `None` if this is the first of its siblings.
    pub fn prev_sibling(&self) -> Option<NodeRef> {
        let (parent, index) = self.parent_and_index()?;
        let index = index.checked_sub(1)?;
        let sibling = parent.0.children.borrow().get(index).cloned();
        sibling
    }

    /// Whether this node has children.
    pub fn has_children(&self) -> bool {
        !self.0.children.borrow().is_empty()
    }

    /// The first child node of this node.
    ///
    /// Returns `None` if this node has no children.
    pub fn first_child(&self) -> Option<NodeRef> {
        self.0.children.borrow().first().cloned()
    }

    /// The last child node of this node.
    ///
    /// Returns `None` if this node has no children.
    pub fn last_child(&self) -> Option<NodeRef> {
        self.0.children.borrow().last().cloned()
    }

    /// Get an iterator through the children of this node.
    pub fn children(&self) -> Children {
        Children::new(self.first_child())
    }

    pub(crate) fn serialize<S>(&self, serializer: &mut S) -> io::Result<()>
    where
        S: Serializer,
    {
        match self.data() {
            NodeData::Element(data) => {
                serializer.start_elem(
                    data.name.clone(),
                    data.attrs.borrow().iter().map(|attr| (&attr.name, &*attr.value)),
                )?;

                for child in self.children() {
                    child.serialize(serializer)?;
                }

                serializer.end_elem(data.name.clone())?;

                Ok(())
            }
            NodeData::Document => {
                for child in self.children() {
                    child.serialize(serializer)?;
                }

                Ok(())
            }
            NodeData::Text(text) => serializer.write_text(&text.borrow()),
            _ => Ok(()),
        }
    }
}

/// An iterator through the children of a node.
///
/// Can be constructed with [`Html::children()`] or [`NodeRef::children()`].
#[derive(Debug, Clone)]
pub struct Children {
    next: Option<NodeRef>,
}

impl Children {
    /// Construct a `Children` starting from the given node.
    fn new(start_node: Option<NodeRef>) -> Self {
        Self { next: start_node }
    }
}

impl Iterator for Children {
    type Item = NodeRef;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next.take()?;
        self.next = next.next_sibling();
        Some(next)
    }
}

impl FusedIterator for Children {}

#[cfg(test)]
mod tests {
    use super::Html;

    #[test]
    fn sanity() {
        let html = "\
            <h1>Title</h1>\
            <div>\
                <p>This is some <em>text</em></p>\
            </div>\
        ";
        assert_eq!(Html::parse(html).to_string(), html);

        assert_eq!(Html::parse("").to_string(), "");
    }
}
