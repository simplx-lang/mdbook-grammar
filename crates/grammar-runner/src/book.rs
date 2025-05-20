use crate::{
  code::{find_rules, parse_code},
  iter::RecursiveIterable,
};
use ecow::EcoString;
use grammar_syntax::{parse, SyntaxNode};
use mdbook::book::Book;
use unscanny::Scanner;

pub fn run(book: &mut Book) {
  let mut pages: Vec<Page> = Vec::new();

  for chapter in book.recur_iter() {
    pages.push(Page {
      href: chapter.path.as_ref().unwrap().to_str().unwrap().into(),
      items: parse_content(chapter.content.clone()),
    });
  }

  let rules = find_rules(&pages);

  let mut parsed_pages = pages.iter().map(|page| {
    page
        .items
        .iter()
        .map(|item| match item {
          | Item::Text(text) => text.clone(),
          | Item::Code(code) => parse_code(&rules, code),
        })
        .collect::<Vec<_>>()
        .join("")
  });

  for chapter in book.recur_iter_mut() {
    chapter.content = parsed_pages.next().unwrap();
  }
}

#[derive(Clone, Debug)]
pub struct Page {
  pub href: EcoString,
  pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub enum Item {
  Text(String),
  Code(SyntaxNode),
}

fn parse_content(content: String) -> Vec<Item> {
  let mut items = Vec::new();
  let mut s = Scanner::new(content.as_str());
  let mut start = s.cursor();

  while !s.done() {
    let mut cs = s;
    let backticks = cs.eat_while('`');
    if backticks.len() >= 3 && cs.eat_if("syntax\n") {
      items.push(Item::Text(s.from(start).to_string()));
      let st = cs.cursor();
      cs.eat_until(backticks);
      items.push(Item::Code(parse(cs.from(st))));
      cs.eat_if(backticks);
      start = cs.cursor();
      s = cs;
    } else {
      s.eat();
    }
  }

  items.push(Item::Text(s.from(start).to_string()));

  items
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::assert_matches::assert_matches;

  #[test]
  fn test_parse_content() {
    let content = r#"
      123123 `123`

      ```syntax
      rule: ;
      ```

      hahahaha

      ```
      123
      ```

      ```syntax
      rule: ;
      ```

      hahaha
    "#;

    let items = parse_content(content.to_string());
    assert_eq!(items.len(), 5);
    assert_matches!(items[0], Item::Text(_));
    assert_matches!(items[1], Item::Code(_));
    assert_matches!(items[2], Item::Text(_));
    assert_matches!(items[3], Item::Code(_));
    assert_matches!(items[4], Item::Text(_));
  }
}
