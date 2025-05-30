use crate::{SyntaxKind, SyntaxNode, lexer::Lexer};
use ecow::{EcoString, eco_format};
use std::ops::{Index, IndexMut};

/// Parse a grammar rule from the input string.
pub fn parse(input: &str) -> SyntaxNode {
    let mut p = Parser::new(input);
    loop {
        p.eat_while(SyntaxKind::is_trivia);

        if p.lexer.done() {
            break;
        }

        rule(&mut p);
    }

    p.finish(SyntaxKind::Root)
}

/// Parse the next rule.
fn rule(p: &mut Parser<'_>) {
    let start = p.marker();

    p.expect([SyntaxKind::Identifier, SyntaxKind::If]);
    p.convert(SyntaxKind::Identifier);
    p.expect(SyntaxKind::Colon);

    let marker = p.marker();
    expression(p);
    p.wrap(marker, SyntaxKind::Definition);

    p.eat_while(SyntaxKind::Action);
    p.expect(SyntaxKind::SemiColon);
    p.hint("consider ending the rule with `;`");

    p.wrap(start, SyntaxKind::Rule);
}

/// Parse an expression greedily.
fn expression(p: &mut Parser<'_>) {
    while item(p, None) {}
}

/// Parse the next item in an expression.
///
/// This will detect the followed repeating indicator automatically.
///
/// If `wrapper` is assigned, wrap the item from the given marker into the given
/// kind before detecting repeating indicator.
fn item(p: &mut Parser, wrapper: Option<(Marker, SyntaxKind)>) -> bool {
    let start = p.marker();

    match p.eat() {
        | SyntaxKind::Meta
        | SyntaxKind::Identifier
        | SyntaxKind::Dot
        | SyntaxKind::Bar => {},

        | SyntaxKind::String => {
            if p.eat_if(SyntaxKind::Dots) {
                p.expect(SyntaxKind::String);
                p.hint("`..` can only connect two string literals");
                p.wrap(start, SyntaxKind::Range);
            }
        },

        | SyntaxKind::Tilde => {
            if !item(p, Some((start, SyntaxKind::Converse))) {
                p.unexpected();
                p.hint(
                    "consider using converse indicator `~` before an \
                     expression",
                )
            }
        },

        | SyntaxKind::LeftParen => {
            let kind = if p.eat_if(SyntaxKind::is_looking) {
                SyntaxKind::Looking
            } else {
                SyntaxKind::Group
            };
            expression(p);
            p.expect(SyntaxKind::RightParen);
            p.wrap(start, kind);
        },

        | SyntaxKind::LeftBrace => {
            p.unexpected();
            p.hint("range should be attached to an expression");
        },
        | SyntaxKind::RightBrace => {
            p.unexpected();
            p.hint("consider starting a range with `{`");
        },

        | SyntaxKind::RightParen
        | SyntaxKind::Action
        | SyntaxKind::End
        | SyntaxKind::SemiColon => {
            p.uneat();
            return false;
        },

        | _ => p.unexpected(),
    }

    if let Some((start, kind)) = wrapper {
        p.wrap(start, kind);
    }

    let start = p.marker();

    if p.eat_if(SyntaxKind::is_prefix) {
        // there is a repeating prefix
        if p.kind() == SyntaxKind::LeftBrace {
            // parse the range
            p.expect(SyntaxKind::Integer);
            if p.eat_if(SyntaxKind::Comma) {
                p.eat_if(SyntaxKind::Integer);
            }
            p.expect(SyntaxKind::RightBrace);
            p.hint("consider closing the range with `}`");
            p.wrap(start, SyntaxKind::BraceIndicator);
        }
        p.eat_if(SyntaxKind::Question);
        p.wrap(start.prev(), SyntaxKind::Repeating);
    }

    true
}

/// Manages parsing a stream of tokens into a tree of [`SyntaxNode`]s.
struct Parser<'s> {
    lexer: Lexer<'s>,
    nodes: Vec<SyntaxNode>,
}

impl<'s> Parser<'s> {
    /// Create a new parser for the given text.
    fn new(text: &'s str) -> Self {
        Self {
            lexer: Lexer::new(text),
            nodes: Vec::new(),
        }
    }

    /// Finish parsing and return the resulting nodes as a single node.
    fn finish(self, kind: SyntaxKind) -> SyntaxNode {
        SyntaxNode::inner(kind, self.nodes)
    }
}

impl Parser<'_> {
    /// Eat tokens until a non-trivia token is found.
    /// Return the kind of the first non-trivia token.
    fn eat(&mut self) -> SyntaxKind {
        loop {
            let node = self.lexer.next();
            let kind = node.kind();
            self.nodes.push(node);
            if !kind.is_trivia() {
                return kind;
            }
        }
    }

    /// Pop the last node and jump the lexer back to its start.
    fn uneat(&mut self) -> SyntaxNode {
        let node = self.nodes.pop().unwrap();
        self.lexer.jump(node.span().start);
        node
    }

    /// Eat the next token if it matches the given pattern.
    fn eat_if(&mut self, pattern: impl Pattern) -> bool {
        if pattern.matches(self.eat()) {
            true
        } else {
            self.uneat();
            false
        }
    }

    /// Eat tokens while the pattern matches.
    fn eat_while(&mut self, pattern: impl Pattern) {
        while pattern.matches(self.eat()) {}
        self.uneat();
    }

    /// Convert the last node to the given kind.
    fn convert(&mut self, kind: SyntaxKind) {
        let node = self.nodes.last_mut().unwrap();
        node.convert_kind(kind);
    }

    /// The kind of the last token.
    fn kind(&self) -> SyntaxKind {
        self.nodes.last().unwrap().kind()
    }

    /// Wrap the nodes after `from` into a new node of the given kind.
    fn wrap(&mut self, from: Marker, kind: SyntaxKind) {
        let to = self.marker().0;
        let from = from.0.min(to);
        let children = self.nodes.split_off(from);
        let node = SyntaxNode::inner(kind, children);
        self.nodes.push(node);
    }

    /// Return the marker pointing to the next node.
    fn marker(&self) -> Marker {
        Marker(self.nodes.len())
    }
}

impl Parser<'_> {
    /// Report an error at the current position.
    fn error(&mut self, message: impl Into<EcoString>) {
        self.nodes.last_mut().unwrap().convert_to_error(message);
    }

    /// Expect the next token to match the given pattern.
    /// If it does not match, report an error.
    fn expect(&mut self, pattern: impl Pattern) -> bool {
        if pattern.matches(self.eat()) {
            true
        } else {
            self.expected(pattern);
            false
        }
    }

    /// Report an error saying that the token is not what is expected.
    fn expected(&mut self, pattern: impl Pattern) {
        self.error(eco_format!(
            "expected {}, found {}",
            pattern.name(),
            self.kind().name()
        ));
    }

    /// Report an error saying that the token is unexpected.
    fn unexpected(&mut self) {
        self.error(eco_format!("unexpected {}", self.kind().name(),));
    }

    /// Add a hint to the last node if it is an error.
    fn hint(&mut self, hint: impl Into<EcoString>) {
        if let Some(node) = self.nodes.last_mut() {
            node.hints(hint);
        }
    }
}

/// Marks a position in the parser's node list.
#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
struct Marker(usize);

impl Marker {
    /// The previous position in the list.
    fn prev(self) -> Self {
        debug_assert!(self.0 > 0);
        Marker(self.0 - 1)
    }
}

impl Index<Marker> for Parser<'_> {
    type Output = SyntaxNode;

    fn index(&self, index: Marker) -> &Self::Output {
        &self.nodes[index.0]
    }
}

impl IndexMut<Marker> for Parser<'_> {
    fn index_mut(&mut self, index: Marker) -> &mut Self::Output {
        &mut self.nodes[index.0]
    }
}

/// A pattern to match a token kind.
trait Pattern {
    fn matches(&self, node: SyntaxKind) -> bool;
    fn name(&self) -> EcoString;
}

impl Pattern for SyntaxKind {
    fn matches(&self, kind: SyntaxKind) -> bool {
        *self == kind
    }

    fn name(&self) -> EcoString {
        SyntaxKind::name(*self).into()
    }
}

impl<const N: usize> Pattern for [SyntaxKind; N] {
    fn matches(&self, kind: SyntaxKind) -> bool {
        self.contains(&kind)
    }

    fn name(&self) -> EcoString {
        self.iter()
            .map(|kind| kind.name())
            .collect::<Vec<_>>()
            .join(" or ")
            .into()
    }
}

impl<F> Pattern for F
where
    F: Fn(SyntaxKind) -> bool,
{
    fn matches(&self, kind: SyntaxKind) -> bool {
        self(kind)
    }

    fn name(&self) -> EcoString {
        panic!("cannot name a function pattern")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::{Debug, Formatter};

    #[derive(Clone, Eq, PartialEq, Hash)]
    struct TestNode {
        kind: SyntaxKind,
        text: EcoString,
        children: Vec<TestNode>,
    }

    impl TestNode {
        fn leaf(kind: SyntaxKind, text: EcoString) -> Self {
            Self {
                kind,
                text,
                children: Vec::new(),
            }
        }

        fn inner(kind: SyntaxKind, children: Vec<TestNode>) -> Self {
            Self {
                kind,
                text: EcoString::new(),
                children,
            }
        }

        fn text(&self) -> String {
            format!(
                "{}{}",
                self.text,
                self.children.iter().map(|c| c.text()).collect::<String>()
            )
        }

        fn test(&self) {
            let text = self.text();
            let actual = parse(&text);
            assert_eq!(actual, *self, "{text}");
        }
    }

    impl Debug for TestNode {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            if self.text.is_empty() {
                f.debug_struct("Inner")
                    .field("kind", &self.kind)
                    .field("children", &self.children)
                    .finish()
            } else {
                f.debug_struct("Leaf")
                    .field("kind", &self.kind)
                    .field("text", &self.text)
                    .finish()
            }
        }
    }

    impl PartialEq<TestNode> for SyntaxNode {
        fn eq(&self, other: &TestNode) -> bool {
            self.kind() == other.kind
                && *self.text() == other.text
                && self.children().len() == other.children.len()
                && self
                    .children()
                    .zip(other.children.iter())
                    .all(|(a, b)| a == b)
        }
    }

    fn default(kind: SyntaxKind) -> &'static str {
        match kind {
            | SyntaxKind::Whitespace => " ",
            | SyntaxKind::Identifier => "identifier",
            | SyntaxKind::String => "\"string\"",
            | SyntaxKind::Integer => "1",
            | SyntaxKind::Meta => "<meta>",
            | SyntaxKind::Operation => " operation ",
            | SyntaxKind::If => "if",
            | SyntaxKind::Colon => ":",
            | SyntaxKind::SemiColon => ";",
            | SyntaxKind::Arrow => "->",
            | SyntaxKind::LeftParen => "(",
            | SyntaxKind::RightParen => ")",
            | SyntaxKind::LeftBrace => "{",
            | SyntaxKind::RightBrace => "}",
            | SyntaxKind::Comma => ",",
            | SyntaxKind::Bar => "|",
            | SyntaxKind::Tilde => "~",
            | SyntaxKind::Dot => ".",
            | SyntaxKind::Question => "?",
            | SyntaxKind::Star => "*",
            | SyntaxKind::Plus => "+",
            | SyntaxKind::Dots => "..",
            | SyntaxKind::LookAheadPos => "?=",
            | SyntaxKind::LookAheadNeg => "?!",
            | SyntaxKind::LookBehindPos => "?<=",
            | SyntaxKind::LookBehindNeg => "?<!",
            | _ => "",
        }
    }

    macro_rules! node {
        {
            $kind:ident => {
                $( $kind2:ident $(=> $tt:tt)? ),* $(,)?
            }
        } => {
            TestNode::inner(SyntaxKind::$kind, vec![
                $(
                    node! {
                        $kind2 $(=> $tt)?
                    }
                ),*
            ])
        };

        { $kind:ident => $text:expr } => {{
            let text = if SyntaxKind::$kind == SyntaxKind::String {
                stringify!($text)
            } else { $text };
            TestNode::leaf(SyntaxKind::$kind, text.into())
        }};

        { $kind:ident } => {
            TestNode::leaf(
                SyntaxKind::$kind,
                default(SyntaxKind::$kind).into()
            )
        };
    }

    macro_rules! test_node {
        { $($tt:tt)* } => {
            node! { $($tt)* }.test()
        }
    }

    #[test]
    fn test_empty() {
        test_node! {
            Root => {}
        }
    }

    #[test]
    fn test_rule_empty() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {},
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_whitespace() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Whitespace,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_line_comment() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Comment => "// comment",
                        Whitespace => "\n",
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_block_comment() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Comment => "/* comment */",
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_identifier() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Identifier,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_string() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        String,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_meta() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Meta,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_if_action() {
        test_node!(
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {},
                    Action => {
                        If,
                        Operation,
                    },
                    SemiColon,
                }
            }
        )
    }

    #[test]
    fn test_arrow_action() {
        test_node!(
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {},
                    Action => {
                        Arrow,
                        Operation,
                    },
                    SemiColon,
                }
            }
        )
    }

    #[test]
    fn test_rule_group() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Group => {
                            LeftParen,
                            Identifier,
                            RightParen,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_lookahead_pos() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Looking => {
                            LeftParen,
                            LookAheadPos,
                            Identifier,
                            RightParen,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_lookahead_neg() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Looking => {
                            LeftParen,
                            LookAheadNeg,
                            Identifier,
                            RightParen,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_lookbehind_pos() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Looking => {
                            LeftParen,
                            LookBehindPos,
                            Identifier,
                            RightParen,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_lookbehind_neg() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Looking => {
                            LeftParen,
                            LookBehindNeg,
                            Identifier,
                            RightParen,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_brace() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            BraceIndicator => {
                                LeftBrace,
                                Integer,
                                Comma,
                                Integer,
                                RightBrace,
                            },
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_question() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Question,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_star() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Star,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_plus() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Plus,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_ungreedy_brace() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            BraceIndicator => {
                                LeftBrace,
                                Integer,
                                Comma,
                                Integer,
                                RightBrace,
                            },
                            Question,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_ungreedy_question() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Question,
                            Question,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_ungreedy_star() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Star,
                            Question,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_prefix_ungreedy_plus() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Repeating => {
                            Identifier,
                            Plus,
                            Question,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_bar() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Identifier,
                        Bar,
                        Identifier,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_converse() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Converse => {
                            Tilde,
                            Identifier,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_dot() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Dot,
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_rule_range() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {
                        Range => {
                            String,
                            Dots,
                            String,
                        },
                    },
                    SemiColon,
                }
            }
        }
    }

    #[test]
    fn test_multi_rules() {
        test_node! {
            Root => {
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {},
                    SemiColon,
                },
                Rule => {
                    Identifier,
                    Colon,
                    Definition => {},
                    SemiColon,
                },
            }
        }
    }

    #[test]
    fn test_mixed_rules() {
        test_node! {
            Root => {
                Whitespace,
                Rule => {
                    Identifier,
                    Whitespace,
                    Colon,
                    Definition => {
                        Repeating => {
                            Converse => {
                                Tilde,
                                Group => {
                                    LeftParen,
                                    Identifier,
                                    Bar,
                                    Identifier,
                                    RightParen,
                                },
                            },
                            BraceIndicator => {
                                LeftBrace,
                                Integer,
                                Whitespace,
                                Comma,
                                Integer,
                                RightBrace,
                            },
                        },
                        Bar,
                        Range => {
                            String,
                            Whitespace,
                            Dots,
                            String,
                        }
                    },
                    Action => { If, Operation },
                    Action => { Arrow, Operation },
                    SemiColon,
                },
                Whitespace,
                Rule => {
                    Identifier,
                    Comment => "/* comment */",
                    Colon,
                    Definition => {
                        Group => {
                            LeftParen,
                            Whitespace,
                            Identifier,
                            Bar,
                            Looking => {
                                LeftParen,
                                LookAheadPos,
                                Repeating => {
                                    Converse => {
                                        Tilde,
                                        Group => {
                                            LeftParen,
                                            Identifier,
                                            Bar,
                                            Identifier,
                                            RightParen,
                                        },
                                    },
                                    Star,
                                    Question,
                                },
                                RightParen,
                            },
                            Whitespace,
                            RightParen
                        },
                    },
                    Action => { If, Operation },
                    SemiColon,
                },
                Whitespace
            }
        }
    }
}
