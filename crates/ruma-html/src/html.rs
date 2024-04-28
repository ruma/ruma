use std::{collections::BTreeSet, fmt, io, iter::FusedIterator};

use as_variant::as_variant;
use html5ever::{
    local_name, namespace_url, ns, parse_fragment,
    serialize::{serialize, Serialize, SerializeOpts, Serializer, TraversalScope},
    tendril::{StrTendril, TendrilSink},
    tree_builder::{NodeOrText, TreeSink},
    Attribute, ParseOpts, QualName,
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
    pub(crate) nodes: Vec<Node>,
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
    pub fn sanitize(&mut self) {
        let config = SanitizerConfig::compat().remove_reply_fallback();
        self.sanitize_with(&config);
    }

    /// Sanitize this HTML according to the given configuration.
    pub fn sanitize_with(&mut self, config: &SanitizerConfig) {
        config.clean(self);
    }

    /// Construct a new `Node` with the given data and add it to this `Html`.
    ///
    /// Returns the index of the new node.
    pub(crate) fn new_node(&mut self, data: NodeData) -> usize {
        self.nodes.push(Node::new(data));
        self.nodes.len() - 1
    }

    /// Append the given node to the given parent in this `Html`.
    ///
    /// The node is detached from its previous position.
    pub(crate) fn append_node(&mut self, parent_id: usize, node_id: usize) {
        self.detach(node_id);

        self.nodes[node_id].parent = Some(parent_id);
        if let Some(last_child) = self.nodes[parent_id].last_child.take() {
            self.nodes[node_id].prev_sibling = Some(last_child);
            self.nodes[last_child].next_sibling = Some(node_id);
        } else {
            self.nodes[parent_id].first_child = Some(node_id);
        }
        self.nodes[parent_id].last_child = Some(node_id);
    }

    /// Insert the given node before the given sibling in this `Html`.
    ///
    /// The node is detached from its previous position.
    pub(crate) fn insert_before(&mut self, sibling_id: usize, node_id: usize) {
        self.detach(node_id);

        self.nodes[node_id].parent = self.nodes[sibling_id].parent;
        self.nodes[node_id].next_sibling = Some(sibling_id);
        if let Some(prev_sibling) = self.nodes[sibling_id].prev_sibling.take() {
            self.nodes[node_id].prev_sibling = Some(prev_sibling);
            self.nodes[prev_sibling].next_sibling = Some(node_id);
        } else if let Some(parent) = self.nodes[sibling_id].parent {
            self.nodes[parent].first_child = Some(node_id);
        }
        self.nodes[sibling_id].prev_sibling = Some(node_id);
    }

    /// Detach the given node from this `Html`.
    pub(crate) fn detach(&mut self, node_id: usize) {
        let (parent, prev_sibling, next_sibling) = {
            let node = &mut self.nodes[node_id];
            (node.parent.take(), node.prev_sibling.take(), node.next_sibling.take())
        };

        if let Some(next_sibling) = next_sibling {
            self.nodes[next_sibling].prev_sibling = prev_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent].last_child = prev_sibling;
        }

        if let Some(prev_sibling) = prev_sibling {
            self.nodes[prev_sibling].next_sibling = next_sibling;
        } else if let Some(parent) = parent {
            self.nodes[parent].first_child = next_sibling;
        }
    }

    /// Get the ID of the root node of the HTML.
    pub(crate) fn root_id(&self) -> usize {
        self.nodes[0].first_child.expect("html should always have a root node")
    }

    /// Get the root node of the HTML.
    pub(crate) fn root(&self) -> &Node {
        &self.nodes[self.root_id()]
    }

    /// Whether the root node of the HTML has children.
    pub fn has_children(&self) -> bool {
        self.root().first_child.is_some()
    }

    /// The first child node of the root node of the HTML.
    ///
    /// Returns `None` if the root node has no children.
    pub fn first_child(&self) -> Option<NodeRef<'_>> {
        self.root().first_child.map(|id| NodeRef::new(self, id))
    }

    /// The last child node of the root node of the HTML .
    ///
    /// Returns `None` if the root node has no children.
    pub fn last_child(&self) -> Option<NodeRef<'_>> {
        self.root().last_child.map(|id| NodeRef::new(self, id))
    }

    /// Iterate through the children of the root node of the HTML.
    pub fn children(&self) -> Children<'_> {
        Children::new(self.first_child())
    }
}

impl Default for Html {
    fn default() -> Self {
        Self { nodes: vec![Node::new(NodeData::Document)] }
    }
}

impl TreeSink for Html {
    type Handle = usize;
    type Output = Self;

    fn finish(self) -> Self::Output {
        self
    }

    fn parse_error(&mut self, msg: std::borrow::Cow<'static, str>) {
        debug!("HTML parse error: {msg}");
    }

    fn get_document(&mut self) -> Self::Handle {
        0
    }

    fn elem_name<'a>(&'a self, target: &'a Self::Handle) -> html5ever::ExpandedName<'a> {
        self.nodes[*target].as_element().expect("not an element").name.expanded()
    }

    fn create_element(
        &mut self,
        name: QualName,
        attrs: Vec<Attribute>,
        _flags: html5ever::tree_builder::ElementFlags,
    ) -> Self::Handle {
        self.new_node(NodeData::Element(ElementData { name, attrs: attrs.into_iter().collect() }))
    }

    fn create_comment(&mut self, _text: StrTendril) -> Self::Handle {
        self.new_node(NodeData::Other)
    }

    fn create_pi(&mut self, _target: StrTendril, _data: StrTendril) -> Self::Handle {
        self.new_node(NodeData::Other)
    }

    fn append(&mut self, parent: &Self::Handle, child: NodeOrText<Self::Handle>) {
        match child {
            NodeOrText::AppendNode(index) => self.append_node(*parent, index),
            NodeOrText::AppendText(text) => {
                // If the previous sibling is also text, add this text to it.
                if let Some(sibling) =
                    self.nodes[*parent].last_child.and_then(|child| self.nodes[child].as_text_mut())
                {
                    sibling.push_tendril(&text);
                } else {
                    let index = self.new_node(NodeData::Text(text));
                    self.append_node(*parent, index);
                }
            }
        }
    }

    fn append_based_on_parent_node(
        &mut self,
        element: &Self::Handle,
        prev_element: &Self::Handle,
        child: NodeOrText<Self::Handle>,
    ) {
        if self.nodes[*element].parent.is_some() {
            self.append_before_sibling(element, child);
        } else {
            self.append(prev_element, child);
        }
    }

    fn append_doctype_to_document(
        &mut self,
        _name: StrTendril,
        _public_id: StrTendril,
        _system_id: StrTendril,
    ) {
    }

    fn get_template_contents(&mut self, target: &Self::Handle) -> Self::Handle {
        *target
    }

    fn same_node(&self, x: &Self::Handle, y: &Self::Handle) -> bool {
        x == y
    }

    fn set_quirks_mode(&mut self, _mode: html5ever::tree_builder::QuirksMode) {}

    fn append_before_sibling(
        &mut self,
        sibling: &Self::Handle,
        new_node: NodeOrText<Self::Handle>,
    ) {
        match new_node {
            NodeOrText::AppendNode(index) => self.insert_before(*sibling, index),
            NodeOrText::AppendText(text) => {
                // If the previous sibling is also text, add this text to it.
                if let Some(prev_text) = self.nodes[*sibling]
                    .prev_sibling
                    .and_then(|prev| self.nodes[prev].as_text_mut())
                {
                    prev_text.push_tendril(&text);
                } else {
                    let index = self.new_node(NodeData::Text(text));
                    self.insert_before(*sibling, index);
                }
            }
        }
    }

    fn add_attrs_if_missing(&mut self, target: &Self::Handle, attrs: Vec<Attribute>) {
        let target = self.nodes[*target].as_element_mut().unwrap();
        target.attrs.extend(attrs);
    }

    fn remove_from_parent(&mut self, target: &Self::Handle) {
        self.detach(*target);
    }

    fn reparent_children(&mut self, node: &Self::Handle, new_parent: &Self::Handle) {
        let mut next_child = self.nodes[*node].first_child;
        while let Some(child) = next_child {
            next_child = self.nodes[child].next_sibling;
            self.append_node(*new_parent, child);
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
                let root = self.root();

                let mut next_child = root.first_child;
                while let Some(child) = next_child {
                    let child = &self.nodes[child];
                    child.serialize(self, serializer)?;
                    next_child = child.next_sibling;
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
pub(crate) struct Node {
    pub(crate) parent: Option<usize>,
    pub(crate) prev_sibling: Option<usize>,
    pub(crate) next_sibling: Option<usize>,
    pub(crate) first_child: Option<usize>,
    pub(crate) last_child: Option<usize>,
    pub(crate) data: NodeData,
}

impl Node {
    /// Constructs a new `Node` with the given data.
    fn new(data: NodeData) -> Self {
        Self {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }

    /// Returns the data of this `Node` if it is an Element (aka an HTML tag).
    pub(crate) fn as_element(&self) -> Option<&ElementData> {
        as_variant!(&self.data, NodeData::Element)
    }

    /// Returns the mutable `ElementData` of this `Node` if it is a `NodeData::Element`.
    pub(crate) fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        as_variant!(&mut self.data, NodeData::Element)
    }

    /// Returns the text content of this `Node`, if it is a `NodeData::Text`.
    fn as_text(&self) -> Option<&StrTendril> {
        as_variant!(&self.data, NodeData::Text)
    }

    /// Returns the mutable text content of this `Node`, if it is a `NodeData::Text`.
    fn as_text_mut(&mut self) -> Option<&mut StrTendril> {
        as_variant!(&mut self.data, NodeData::Text)
    }
}

impl Node {
    pub(crate) fn serialize<S>(&self, fragment: &Html, serializer: &mut S) -> io::Result<()>
    where
        S: Serializer,
    {
        match &self.data {
            NodeData::Element(data) => {
                serializer.start_elem(
                    data.name.clone(),
                    data.attrs.iter().map(|attr| (&attr.name, &*attr.value)),
                )?;

                let mut next_child = self.first_child;
                while let Some(child) = next_child {
                    let child = &fragment.nodes[child];
                    child.serialize(fragment, serializer)?;
                    next_child = child.next_sibling;
                }

                serializer.end_elem(data.name.clone())?;

                Ok(())
            }
            NodeData::Document => {
                let mut next_child = self.first_child;
                while let Some(child) = next_child {
                    let child = &fragment.nodes[child];
                    child.serialize(fragment, serializer)?;
                    next_child = child.next_sibling;
                }

                Ok(())
            }
            NodeData::Text(text) => serializer.write_text(text),
            _ => Ok(()),
        }
    }
}

/// The data of a `Node`.
#[derive(Debug, Clone)]
#[allow(clippy::exhaustive_enums)]
pub enum NodeData {
    /// The root node of the `Html`.
    Document,

    /// A text node.
    Text(StrTendril),

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
    pub attrs: BTreeSet<Attribute>,
}

impl ElementData {
    /// Convert this element data to typed data as [suggested by the Matrix Specification][spec].
    ///
    /// [spec]: https://spec.matrix.org/latest/client-server-api/#mroommessage-msgtypes
    #[cfg(feature = "matrix")]
    pub fn to_matrix(&self) -> matrix::MatrixElementData {
        matrix::MatrixElementData::parse(&self.name, &self.attrs)
    }
}

/// A reference to an HTML node.
#[derive(Debug, Clone, Copy)]
#[non_exhaustive]
pub struct NodeRef<'a> {
    /// The `Html` struct containing the nodes.
    pub(crate) html: &'a Html,
    /// The referenced node.
    pub(crate) node: &'a Node,
}

impl<'a> NodeRef<'a> {
    /// Construct a new `NodeRef` for the given HTML and node ID.
    fn new(html: &'a Html, id: usize) -> Self {
        Self { html, node: &html.nodes[id] }
    }

    /// Construct a new `NodeRef` from the same HTML as this node with the given node ID.
    fn with_id(&self, id: usize) -> Self {
        let html = self.html;
        Self::new(html, id)
    }

    /// The data of the node.
    pub fn data(&self) -> &'a NodeData {
        &self.node.data
    }

    /// Returns the data of this node if it is a `NodeData::Element`.
    pub fn as_element(&self) -> Option<&'a ElementData> {
        self.node.as_element()
    }

    /// Returns the text content of this node, if it is a `NodeData::Text`.
    pub fn as_text(&self) -> Option<&'a StrTendril> {
        self.node.as_text()
    }

    /// The parent node of this node.
    ///
    /// Returns `None` if the parent is the root node.
    pub fn parent(&self) -> Option<NodeRef<'a>> {
        let parent_id = self.node.parent?;

        // We don't want users to be able to navigate to the root.
        if parent_id == self.html.root_id() {
            return None;
        }

        Some(self.with_id(parent_id))
    }

    /// The next sibling node of this node.
    ///
    /// Returns `None` if this is the last of its siblings.
    pub fn next_sibling(&self) -> Option<NodeRef<'a>> {
        Some(self.with_id(self.node.next_sibling?))
    }

    /// The previous sibling node of this node.
    ///
    /// Returns `None` if this is the first of its siblings.
    pub fn prev_sibling(&self) -> Option<NodeRef<'a>> {
        Some(self.with_id(self.node.prev_sibling?))
    }

    /// Whether this node has children.
    pub fn has_children(&self) -> bool {
        self.node.first_child.is_some()
    }

    /// The first child node of this node.
    ///
    /// Returns `None` if this node has no children.
    pub fn first_child(&self) -> Option<NodeRef<'a>> {
        Some(self.with_id(self.node.first_child?))
    }

    /// The last child node of this node.
    ///
    /// Returns `None` if this node has no children.
    pub fn last_child(&self) -> Option<NodeRef<'a>> {
        Some(self.with_id(self.node.last_child?))
    }

    /// Get an iterator through the children of this node.
    pub fn children(&self) -> Children<'a> {
        Children::new(self.first_child())
    }
}

/// An iterator through the children of a node.
///
/// Can be constructed with [`Html::children()`] or [`NodeRef::children()`].
#[derive(Debug, Clone, Copy)]
pub struct Children<'a> {
    next: Option<NodeRef<'a>>,
}

impl<'a> Children<'a> {
    /// Construct a `Children` starting from the given node.
    fn new(start_node: Option<NodeRef<'a>>) -> Self {
        Self { next: start_node }
    }
}

impl<'a> Iterator for Children<'a> {
    type Item = NodeRef<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.next?;
        self.next = next.next_sibling();
        Some(next)
    }
}

impl<'a> FusedIterator for Children<'a> {}

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
