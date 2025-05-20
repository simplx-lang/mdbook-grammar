use crate::{SyntaxError, SyntaxKind, SyntaxNode};
use ecow::{EcoString, eco_format};
use unscanny::Scanner;

pub struct Lexer<'s> {
  s: Scanner<'s>,
  error: Option<SyntaxError>,
}

impl<'s> Lexer<'s> {
  pub fn new(text: &'s str) -> Self {
    Self {
      s: Scanner::new(text),
      error: None,
    }
  }

  pub fn jump(&mut self, target: usize) {
    self.s.jump(target);
  }

  pub fn done(&self) -> bool {
    self.s.done()
  }
}

impl Lexer<'_> {
  fn error(&mut self, message: impl Into<EcoString>) -> SyntaxKind {
    self.error = Some(SyntaxError::new(message));
    SyntaxKind::Error
  }

  fn hint(&mut self, hint: impl Into<EcoString>) {
    if let Some(error) = &mut self.error {
      error.hint(hint);
    }
  }
}

impl Lexer<'_> {
  pub fn next(&mut self) -> SyntaxNode {
    debug_assert!(self.error.is_none());
    let start = self.s.cursor();

    let kind = match self.s.eat() {
      | Some(c) if c.is_whitespace() => self.whitespace(),
      | Some('/') if self.s.eat_if('/') => self.line_comment(),
      | Some('/') if self.s.eat_if('*') => self.block_comment(),
      | Some('*') if self.s.eat_if('/') => {
        self.error("unexpected end of block comment");
        self.hint("consider opening the block comment with `/*`");
        SyntaxKind::Error
      },
      | None => SyntaxKind::End,

      | Some('"') => self.string(),
      | Some(c) if c.is_numeric() => {
        self.s.eat_while(char::is_numeric);
        SyntaxKind::Integer
      },
      | Some('<') => {
        self.s.eat_until('>');
        if self.s.eat().is_none() {
          self.error("unclosed meta");
          self.hint("consider closing the meta with `>`");
          SyntaxKind::Error
        } else {
          SyntaxKind::Meta
        }
      },
      | Some(c) if is_id_start(c) => return self.identifier(start),
      | Some('-') if self.s.eat_if('>') => {
        return self.action(start, SyntaxKind::Arrow);
      },

      | Some(':') => SyntaxKind::Colon,
      | Some(';') => SyntaxKind::SemiColon,
      | Some('(') => SyntaxKind::LeftParen,
      | Some(')') => SyntaxKind::RightParen,
      | Some('{') => SyntaxKind::LeftBrace,
      | Some('}') => SyntaxKind::RightBrace,
      | Some(',') => SyntaxKind::Comma,
      | Some('|') => SyntaxKind::Bar,
      | Some('~') => SyntaxKind::Tilde,
      | Some('.') if self.s.eat_if('.') => SyntaxKind::Dots,
      | Some('.') => SyntaxKind::Dot,
      | Some('*') => SyntaxKind::Star,
      | Some('+') => SyntaxKind::Plus,
      | Some('?') if self.s.eat_if('=') => SyntaxKind::LookAheadPos,
      | Some('?') if self.s.eat_if('!') => SyntaxKind::LookAheadNeg,
      | Some('?') if self.s.eat_if("<=") => SyntaxKind::LookBehindPos,
      | Some('?') if self.s.eat_if("<!") => SyntaxKind::LookBehindNeg,
      | Some('?') => SyntaxKind::Question,

      | Some(c) => self.error(eco_format!("unexpected character `{c}`")),
    };

    if let Some(error) = self.error.take() {
      SyntaxNode::error(error, self.s.from(start), start..self.s.cursor())
    } else {
      SyntaxNode::leaf(kind, self.s.from(start), start..self.s.cursor())
    }
  }

  fn whitespace(&mut self) -> SyntaxKind {
    self.s.eat_whitespace();
    SyntaxKind::Whitespace
  }

  fn line_comment(&mut self) -> SyntaxKind {
    self.s.eat_until(is_newline);
    SyntaxKind::Comment
  }

  fn block_comment(&mut self) -> SyntaxKind {
    while let Some(c) = self.s.eat() {
      if c == '*' && self.s.eat() == Some('/') {
        return SyntaxKind::Comment;
      }
    }
    self.error("unclosed block comment");
    self.hint("consider closing the block comment with `*/`");
    SyntaxKind::Error
  }

  fn string(&mut self) -> SyntaxKind {
    while let Some(c) = self.s.eat() {
      if c == '"' {
        return SyntaxKind::String;
      } else if c == '\\' {
        if let Some(next) = self.s.eat() {
          match next {
            | 'n' | 'r' | 't' | 'b' | 'f' | '\\' | '"' => {},
            | 'u' => {
              let unicode = if self.s.eat_if('{') {
                let unicode = self.s.eat_while(char::is_alphanumeric);
                if !self.s.eat_if('}') {
                  self.error("unclosed unicode escape");
                  self.hint("consider closing the unicode escape with `}`");
                  continue;
                }
                unicode
              } else {
                let start = self.s.cursor();
                for _ in 0..4 {
                  if self.s.eat().is_none() {
                    break;
                  }
                }
                self.s.from(start)
              };

              if u64::from_str_radix(unicode, 16).is_err() {
                self.error("invalid unicode escape");
                self.hint("unicode must be a hex number");
              }
            },
            | _ => {
              self.error("invalid escape sequence");
            },
          }
        }
      }
    }

    self.error("unclosed string literal");
    self.hint("consider closing the string literal with `\"`");
    SyntaxKind::Error
  }

  fn identifier(&mut self, start: usize) -> SyntaxNode {
    self.s.eat_while(is_id_continue);
    let text = self.s.from(start);

    if text == "if" {
      self.action(start, SyntaxKind::If)
    } else {
      SyntaxNode::leaf(SyntaxKind::Identifier, text, start..self.s.cursor())
    }
  }

  fn action(&mut self, start: usize, kind: SyntaxKind) -> SyntaxNode {
    let text = self.s.from(start);
    let cursor = self.s.cursor();

    if kind == SyntaxKind::Arrow {
      self.s.eat_until(';');
    } else {
      while !self.s.done() && !self.s.at(';') && !self.s.at("->") {
        self.s.eat();
      }
    }

    let action = self.s.from(cursor);

    SyntaxNode::inner(SyntaxKind::Action, vec![
      SyntaxNode::leaf(kind, text, start..cursor),
      SyntaxNode::leaf(SyntaxKind::Operation, action, cursor..self.s.cursor()),
    ])
  }
}

/// Check if the character is a newline.
#[inline]
fn is_newline(c: char) -> bool {
  matches!(
    c,
    // Line Feed, Vertical Tab, Form Feed, Carriage Return.
    '\n' | '\x0B' | '\x0C' | '\r' |
        // Next Line, Line Separator, Paragraph Separator.
        '\u{0085}' | '\u{2028}' | '\u{2029}'
  )
}

/// Check if the character is a valid identifier start character.
#[inline]
fn is_id_start(c: char) -> bool {
  c.is_ascii_alphabetic() || c == '_'
}

/// Check if the character is a valid identifier continuation character.
#[inline]
fn is_id_continue(c: char) -> bool {
  c.is_ascii_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
  use super::*;

  macro_rules! test_lexer {
    ($kind:ident, $next:expr, $more:expr) => {
      let next = $next.clone();
      let node = Lexer::new(format!("{next}{}", $more).as_str()).next();
      assert_eq!(node.kind(), SyntaxKind::$kind);
      assert_eq!(*node.span(), (0..next.len()));
    };

    ($kind:ident, $next:expr) => {
      test_lexer!($kind, $next, "");
    };
  }

  #[test]
  fn test_whitespace() {
    test_lexer!(Whitespace, "  \n  ", "123");
  }

  #[test]
  fn test_line_comment() {
    test_lexer!(Comment, "// comment", "\n123");
  }

  #[test]
  fn test_block_comment() {
    test_lexer!(Comment, "/* comment \n comment */", "123");
  }

  #[test]
  fn test_block_comment_unopened() {
    test_lexer!(Error, "*/", "123");
  }

  #[test]
  fn test_block_comment_unclosed() {
    test_lexer!(Error, "/* comment");
  }

  #[test]
  fn test_end() {
    test_lexer!(End, "");
  }

  #[test]
  fn test_string() {
    test_lexer!(String, r#""str\u{123abc}\n\f123""#, "123");
  }

  #[test]
  fn test_string_invalid_escape() {
    test_lexer!(Error, r#""\a""#);
  }

  #[test]
  fn test_string_invalid_unicode() {
    test_lexer!(Error, r#""\u{xyz}""#);
  }

  #[test]
  fn test_string_unclosed_unicode() {
    test_lexer!(Error, r#""\u{123abchahahaha""#);
  }

  #[test]
  fn test_string_unclosed() {
    test_lexer!(Error, r#""hahahaha"#);
  }

  #[test]
  fn test_integer() {
    test_lexer!(Integer, "123", "abc");
  }

  #[test]
  fn test_identifier() {
    test_lexer!(Identifier, "abc_123_haha", "-123");
  }

  #[test]
  fn test_meta() {
    test_lexer!(Meta, "<if1 \n@$%/\\()[]{}:;>", "123");
  }

  #[test]
  fn test_if() {
    test_lexer!(Action, "if hahahaha", "-> 123123;");
    test_lexer!(Action, "if hahahaha", ";");
    test_lexer!(Action, "if hahahaha");
  }

  #[test]
  fn test_arrow() {
    test_lexer!(Action, "-> hahahaha", ";");
    test_lexer!(Action, "-> hahahaha");
  }

  #[test]
  fn test_symbol() {
    for symbol in [
      ":", ";", "(", ")", "{", "}", ",", "|", "~", ".", "?", "*", "+", "..",
      "?=", "?!", "?<=", "?<!", "?",
    ] {
      let node = Lexer::new(format!("{symbol}abc123").as_str()).next();
      assert!(node.kind().is_operator());
      assert_eq!(*node.span(), 0..symbol.len());
      assert_eq!(node.text(), symbol);
    }
  }

  #[test]
  fn test_unexpected() {
    test_lexer!(Error, "%");
  }
}
