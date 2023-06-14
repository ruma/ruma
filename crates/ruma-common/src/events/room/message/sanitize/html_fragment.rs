use std::{collections::BTreeSet, fmt, io};

use html5ever::{
    local_name, namespace_url, ns, parse_fragment,
    serialize::{serialize, Serialize, SerializeOpts, Serializer, TraversalScope},
    tendril::{StrTendril, TendrilSink},
    tree_builder::{NodeOrText, TreeSink},
    Attribute, ParseOpts, QualName,
};
use tracing::debug;

/// An HTML fragment.
///
/// To get the serialized HTML, use its `Display` implementation.
#[derive(Debug)]
pub(crate) struct Fragment {
    pub nodes: Vec<Node>,
}

impl Fragment {
    /// Construct a new `Fragment` by parsing the given HTML.
    pub fn parse_html(html: &str) -> Self {
        let sink = Self::default();
        let mut parser = parse_fragment(
            sink,
            ParseOpts::default(),
            QualName::new(None, ns!(html), local_name!("div")),
            Vec::new(),
        );
        parser.process(html.into());
        parser.finish()
    }

    /// Construct a new `Node` with the given data and add it to this `Fragment`.
    ///
    /// Returns the index of the new node.
    pub fn new_node(&mut self, data: NodeData) -> usize {
        self.nodes.push(Node::new(data));
        self.nodes.len() - 1
    }

    /// Append the given node to the given parent in this `Fragment`.
    ///
    /// The node is detached from its previous position.
    pub fn append_node(&mut self, parent_id: usize, node_id: usize) {
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

    /// Insert the given node before the given sibling in this `Fragment`.
    ///
    /// The node is detached from its previous position.
    pub fn insert_before(&mut self, sibling_id: usize, node_id: usize) {
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

    /// Detach the given node from this `Fragment`.
    pub fn detach(&mut self, node_id: usize) {
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
}

impl Default for Fragment {
    fn default() -> Self {
        Self { nodes: vec![Node::new(NodeData::Document)] }
    }
}

impl TreeSink for Fragment {
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
        target.attrs.extend(attrs.into_iter());
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

impl Serialize for Fragment {
    fn serialize<S>(&self, serializer: &mut S, traversal_scope: TraversalScope) -> io::Result<()>
    where
        S: Serializer,
    {
        match traversal_scope {
            TraversalScope::IncludeNode => {
                let root = self.nodes[0].first_child.unwrap();

                let mut next_child = self.nodes[root].first_child;
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

impl fmt::Display for Fragment {
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
pub(crate) struct Node {
    pub parent: Option<usize>,
    pub prev_sibling: Option<usize>,
    pub next_sibling: Option<usize>,
    pub first_child: Option<usize>,
    pub last_child: Option<usize>,
    pub data: NodeData,
}

impl Node {
    /// Constructs a new `Node` with the given data.
    pub fn new(data: NodeData) -> Self {
        Self {
            parent: None,
            prev_sibling: None,
            next_sibling: None,
            first_child: None,
            last_child: None,
            data,
        }
    }

    /// Returns the `ElementData` of this `Node` if it is a `NodeData::Element`.
    pub fn as_element(&self) -> Option<&ElementData> {
        match &self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the mutable `ElementData` of this `Node` if it is a `NodeData::Element`.
    pub fn as_element_mut(&mut self) -> Option<&mut ElementData> {
        match &mut self.data {
            NodeData::Element(data) => Some(data),
            _ => None,
        }
    }

    /// Returns the mutable text content of this `Node`, if it is a `NodeData::Text`.
    pub fn as_text_mut(&mut self) -> Option<&mut StrTendril> {
        match &mut self.data {
            NodeData::Text(data) => Some(data),
            _ => None,
        }
    }
}

impl Node {
    pub fn serialize<S>(&self, fragment: &Fragment, serializer: &mut S) -> io::Result<()>
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
#[derive(Debug)]
pub(crate) enum NodeData {
    /// The root node of the `Fragment`.
    Document,

    /// A text node.
    Text(StrTendril),

    /// An HTML element (aka a tag).
    Element(ElementData),

    /// Other types (comment, processing instruction, …).
    Other,
}

/// The data of an HTML element.
#[derive(Debug)]
pub(crate) struct ElementData {
    /// The qualified name of the element.
    pub name: QualName,

    /// The attributes of the element.
    pub attrs: BTreeSet<Attribute>,
}

#[cfg(test)]
mod tests {
    use super::Fragment;

    #[test]
    fn sanity() {
        let html = "\
            <h1>Title</h1>\
            <div>\
                <p>This is some <em>text</em></p>\
            </div>\
        ";
        assert_eq!(Fragment::parse_html(html).to_string(), html);

        assert_eq!(Fragment::parse_html("").to_string(), "");
    }
}
