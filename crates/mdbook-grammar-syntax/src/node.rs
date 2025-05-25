use crate::SyntaxKind;
use ecow::{EcoString, EcoVec};
use std::{
    fmt::{Debug, Formatter},
    ops::Range,
};

/// A node in the untyped syntax tree.
#[derive(Clone, Eq, PartialEq, Hash)]
pub struct SyntaxNode(Repr);

impl SyntaxNode {
    /// Create a new leaf node.
    pub fn leaf(
        kind: SyntaxKind,
        text: impl Into<EcoString>,
        span: Range<usize>,
    ) -> Self {
        Self(Repr::Leaf(LeafNode::new(kind, text, span)))
    }

    /// Create a new inner node.
    pub fn inner(kind: SyntaxKind, children: Vec<SyntaxNode>) -> Self {
        Self(Repr::Inner(InnerNode::new(kind, children)))
    }

    /// Create a new error node.
    pub fn error(
        error: SyntaxError,
        text: impl Into<EcoString>,
        span: Range<usize>,
    ) -> Self {
        Self(Repr::Error(ErrorNode {
            text: text.into(),
            span,
            error,
        }))
    }

    /// The kind of the node.
    pub fn kind(&self) -> SyntaxKind {
        match &self.0 {
            | Repr::Leaf(node) => node.kind,
            | Repr::Inner(node) => node.kind,
            | Repr::Error(_) => SyntaxKind::Error,
        }
    }

    /// The text of the node.
    pub fn text(&self) -> &EcoString {
        static EMPTY: EcoString = EcoString::new();
        match &self.0 {
            | Repr::Leaf(node) => &node.text,
            | Repr::Inner(_) => &EMPTY,
            | Repr::Error(node) => &node.text,
        }
    }

    /// The span of the node.
    pub fn span(&self) -> &Range<usize> {
        match &self.0 {
            | Repr::Leaf(node) => &node.span,
            | Repr::Inner(node) => &node.span,
            | Repr::Error(node) => &node.span,
        }
    }

    /// The children of this node.
    pub fn children(&self) -> std::slice::Iter<'_, SyntaxNode> {
        match &self.0 {
            | Repr::Leaf(_) => [].iter(),
            | Repr::Inner(node) => node.children.iter(),
            | Repr::Error(_) => [].iter(),
        }
    }

    /// Whether this node or its children contains an error.
    pub fn erroneous(&self) -> bool {
        match &self.0 {
            | Repr::Leaf(_) => false,
            | Repr::Inner(node) => node.erroneous,
            | Repr::Error(_) => true,
        }
    }

    /// Add a hint to the error node.
    pub fn hints(&mut self, hint: impl Into<EcoString>) {
        if let Repr::Error(node) = &mut self.0 {
            node.error.hint(hint);
        }
    }

    /// Get the error node if this is an error node.
    pub fn as_error(&self) -> Option<&SyntaxError> {
        if let Repr::Error(node) = &self.0 {
            Some(&node.error)
        } else {
            None
        }
    }
}

impl SyntaxNode {
    pub fn convert_kind(&mut self, kind: SyntaxKind) {
        match &mut self.0 {
            | Repr::Leaf(node) => node.kind = kind,
            | Repr::Inner(node) => node.kind = kind,
            | Repr::Error(_) => {},
        }
    }

    pub fn convert_to_error(&mut self, message: impl Into<EcoString>) {
        if matches!(self.0, Repr::Error(_)) {
            return;
        }
        self.0 = Repr::Error(ErrorNode {
            text: self.text().clone(),
            span: self.span().clone(),
            error: SyntaxError::new(message),
        });
    }
}

#[derive(Clone, Eq, PartialEq, Hash)]
enum Repr {
    Leaf(LeafNode),
    Inner(InnerNode),
    Error(ErrorNode),
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct LeafNode {
    kind: SyntaxKind,
    text: EcoString,
    span: Range<usize>,
}

impl LeafNode {
    fn new(
        kind: SyntaxKind,
        text: impl Into<EcoString>,
        span: Range<usize>,
    ) -> Self {
        debug_assert!(!kind.is_error());

        Self {
            kind,
            text: text.into(),
            span,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct InnerNode {
    kind: SyntaxKind,
    span: Range<usize>,
    erroneous: bool,
    children: Vec<SyntaxNode>,
}

impl InnerNode {
    fn new(kind: SyntaxKind, children: Vec<SyntaxNode>) -> Self {
        debug_assert!(!kind.is_error());

        let erroneous = children.iter().any(|node| node.erroneous());
        let start = children.first().map_or(0, |child| child.span().start);
        let end = children.last().map_or(0, |child| child.span().end);

        Self {
            kind,
            span: start..end,
            erroneous,
            children,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
struct ErrorNode {
    text: EcoString,
    span: Range<usize>,
    error: SyntaxError,
}

/// A syntactical error.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct SyntaxError {
    pub message: EcoString,
    pub hints: EcoVec<EcoString>,
}

impl SyntaxError {
    /// Create a new syntax error.
    pub fn new(message: impl Into<EcoString>) -> Self {
        Self {
            message: message.into(),
            hints: EcoVec::new(),
        }
    }

    /// Add a hint to the error.
    pub fn hint(&mut self, hint: impl Into<EcoString>) {
        self.hints.push(hint.into());
    }
}

impl Debug for SyntaxNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.0 {
            | Repr::Leaf(leaf) => f
                .debug_struct("Leaf")
                .field("kind", &leaf.kind)
                .field("text", &leaf.text)
                .finish(),
            | Repr::Inner(inner) => f
                .debug_struct("Inner")
                .field("kind", &inner.kind)
                .field("children", &inner.children)
                .finish(),
            | Repr::Error(error) => f
                .debug_struct("Error")
                .field("error", &error.error)
                .finish(),
        }
    }
}
