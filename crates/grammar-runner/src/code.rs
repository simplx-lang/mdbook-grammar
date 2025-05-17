use crate::book::{Item, Page};
use ecow::EcoString;
use grammar_syntax::{SyntaxKind, SyntaxNode};
use html_escape::encode_safe;
use std::collections::HashMap;

type Rules = HashMap<EcoString, Vec<EcoString>>;

pub fn find_rules(pages: &Vec<Page>) -> Rules {
    let mut rules: Rules = HashMap::new();

    for page in pages {
        for item in &page.items {
            if let Item::Code(code) = item {
                debug_assert_eq!(code.kind(), SyntaxKind::Root);
                for node in code.children() {
                    if node.kind() == SyntaxKind::Rule && !node.erroneous() {
                        for sub in node.children() {
                            if sub.kind() == SyntaxKind::Identifier {
                                let name = sub.text();
                                let href = format!("{}#{}", page.href, rule_hash(name));
                                if let Some(v) = rules.get_mut(name) {
                                    v.push(href.into());
                                } else {
                                    rules.insert(name.into(), vec![href.into()]);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    rules
}

pub(crate) fn parse_code(rules: &Rules, code: &SyntaxNode) -> String {
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

    format!("<pre><code class=\"hljs\">{content}</code></pre>")
}

fn parse_rule(rules: &Rules, rule: &SyntaxNode) -> String {
    debug_assert_eq!(rule.kind(), SyntaxKind::Rule);
    debug_assert!(!rule.erroneous());

    let name = rule
        .children()
        .find(|n| n.kind() == SyntaxKind::Identifier)
        .unwrap()
        .text();

    format!(
        "<span rule=\"{name}\">\
        <a name=\"#{name}\"></a>\
        {content}</span>",
        name = rule_hash(name),
        content = wrap(rules, rule)
    )
}

pub fn wrap(rules: &Rules, node: &SyntaxNode) -> String {
    let cls = match node.kind() {
        SyntaxKind::Error => return wrap_error(node),
        SyntaxKind::Comment => "hljs-comment",
        SyntaxKind::Whitespace => return node.text().into(),
        SyntaxKind::Identifier => return wrap_identifier(rules, node),
        SyntaxKind::String => "hljs-string",
        SyntaxKind::Integer => "hljs-number",
        SyntaxKind::Meta => "hljs-meta",
        SyntaxKind::Operation => "hljs-quote",
        SyntaxKind::If => "hljs-keyword",
        k if k.is_symbol() => return node.text().into(),
        _ => {
            return node
                .children()
                .map(|n| wrap(rules, n))
                .collect::<Vec<_>>()
                .join("");
        }
    };

    let text = node.text();
    format!(
        "<span class=\"{cls}\">{text}</span>",
        text = encode_safe(text)
    )
}

fn wrap_identifier(rules: &Rules, rule: &SyntaxNode) -> String {
    debug_assert_eq!(rule.kind(), SyntaxKind::Identifier);

    let name = rule.text();
    if let Some(hrefs) = rules.get(name) {
        if hrefs.len() > 1 {
            let message = format!(
                "Found definitions for rule {name} in these sources:\n{hrefs}",
                hrefs = hrefs.join("\n")
            );
            format!(
                "<span class=\"hljs-deletion hljs-strong hljs-emphasis\">\
                <a onclick=\"alert('{message}')\">\
                {name}</a></span>",
                message = message.escape_default(),
            )
        } else {
            format!(
                "<a href=\"{href}\">\
                <span class=\"hljs-name hljs-strong\">\
                {name}</span></a>",
                href = hrefs[0],
            )
        }
    } else {
        format!(
            "<span class=\"hljs-deletion hljs-strong hljs-emphasis\">\
            <a onclick=\"alert('Error: rule {name} undefined')\">\
            {name}\
            </a></span>"
        )
    }
}

fn wrap_error(error: &SyntaxNode) -> String {
    debug_assert_eq!(error.kind(), SyntaxKind::Error);

    let text = {
        let text = error.text();
        if text.trim().is_empty() {
            &"[error]".into()
        } else {
            text
        }
    };
    let error = error.as_error().unwrap();
    let mut message = error.message.clone();
    for hint in &error.hints {
        message.push_str(&format!("\n    {hint}"));
    }

    format!(
        "<span class=\"hljs-deletion\">\
        <a onclick=\"alert('Error: {message}')\">\
        {code}</a></span>",
        message = message.escape_default(),
        code = encode_safe(text),
    )
}

#[inline]
pub fn rule_hash(name: impl ToString) -> String {
    format!("syntax-rule-{name}", name = name.to_string())
}
