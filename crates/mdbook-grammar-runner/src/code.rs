use crate::book::{Item, Page};
use ecow::EcoString;
use html_escape::encode_safe;
use mdbook_grammar_syntax::{SyntaxError, SyntaxKind, SyntaxNode};
use std::collections::HashMap;

type Rules = HashMap<EcoString, EcoString>;

pub fn find_rules(pages: &Vec<Page>, root: &str) -> Rules {
    let mut rules: Rules = HashMap::new();

    for page in pages {
        for item in &page.items {
            if let Item::Code(code) = item {
                // Find rule definitions in code blocks.
                debug_assert_eq!(code.kind(), SyntaxKind::Root);

                for node in code.children() {
                    if node.kind() == SyntaxKind::Rule && !node.erroneous() {
                        // Found a rule definition.
                        let Some(name) = node
                            .children()
                            .find(|n| n.kind() == SyntaxKind::Identifier)
                            .map(SyntaxNode::text)
                            .filter(|name| !name.starts_with('_'))
                        else {
                            continue;
                        };

                        let href =
                            format!("{root}{}#{}", page.href, rule_hash(name));
                        rules.insert(name.into(), href.into());
                    }
                }
            }
        }
    }

    rules
}

pub fn parse_code(rules: &Rules, code: &SyntaxNode) -> String {
    debug_assert_eq!(code.kind(), SyntaxKind::Root);

    let content = code
        .children()
        .map(|node| {
            if node.kind() == SyntaxKind::Rule && !node.erroneous() {
                parse_rule(rules, node)
            } else {
                wrap(rules, node)
            }
        })
        .collect::<Vec<_>>()
        .join("");

    format!("<pre><code class=\"syntax\">{content}</code></pre>")
}

fn parse_rule(rules: &Rules, rule: &SyntaxNode) -> String {
    debug_assert_eq!(rule.kind(), SyntaxKind::Rule);
    debug_assert!(!rule.erroneous());

    let name = rule
        .children()
        .find(|n| n.kind() == SyntaxKind::Identifier)
        .unwrap()
        .text();

    if name.starts_with('_') {
        // Ignored rule.
        wrap(rules, rule)
    } else {
        format!(
            "<span class=\"syntax-rule\" rule=\"{name}\"><a \
             name=\"{name}\"></a>{content}</span>",
            name = rule_hash(name),
            content = wrap(rules, rule)
        )
    }
}

pub fn wrap(rules: &Rules, node: &SyntaxNode) -> String {
    let cls = match node.kind() {
        | SyntaxKind::Error => return wrap_error(node),
        | SyntaxKind::Comment => "comment",
        | SyntaxKind::Whitespace => return node.text().into(),
        | SyntaxKind::Identifier => return wrap_identifier(rules, node),
        | SyntaxKind::String => "string",
        | SyntaxKind::Integer => "integer",
        | SyntaxKind::Meta => "meta",
        | SyntaxKind::Operation => "action",
        | SyntaxKind::If => "keyword",
        | k if k.is_operator() => "operator",
        | _ => {
            return node
                .children()
                .map(|n| wrap(rules, n))
                .collect::<Vec<_>>()
                .join("");
        },
    };

    wrap_node_raw(node.text(), cls)
}

fn wrap_identifier(rules: &Rules, rule: &SyntaxNode) -> String {
    debug_assert_eq!(rule.kind(), SyntaxKind::Identifier);

    let name = rule.text();
    if let Some(href) = rules.get(name) {
        format!(
            "<a class=\"syntax-link\" href=\"{href}\">{content}</a>",
            content = wrap_node_raw(name, "identifier"),
        )
    } else {
        wrap_node_raw(name, "identifier")
    }
}

fn wrap_error(error: &SyntaxNode) -> String {
    debug_assert_eq!(error.kind(), SyntaxKind::Error);
    wrap_error_raw(error.text(), error.as_error().unwrap())
}

fn wrap_node_raw(code: &str, cls: &str) -> String {
    format!(
        "<span class=\"syntax-{cls}\">{text}</span>",
        cls = cls,
        text = encode_safe(code)
    )
}

fn wrap_error_raw(code: &str, error: &SyntaxError) -> String {
    let text = {
        let text = code;
        if text.trim().is_empty() {
            "[error]"
        } else {
            text
        }
    };

    let message = error.message.escape_default();
    let hints = error
        .hints
        .iter()
        .map(|hint| format!("\"{}\"", hint.escape_default()))
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "<span class=\"syntax-error\" message=\"{message}\" \
         hints=\"[{hints}]\">{text}</span>",
        hints = encode_safe(&hints),
    )
}

#[inline]
pub fn rule_hash(name: impl ToString) -> String {
    format!("syntax-rule-{name}", name = name.to_string())
}
