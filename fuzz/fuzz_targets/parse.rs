#![no_main]

use libfuzzer_sys::fuzz_target;
use mdbook_grammar_syntax::parse;

fuzz_target!(|data: &str| {
  parse(data);
});
