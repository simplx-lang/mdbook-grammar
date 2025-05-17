#![no_main]

use grammar_syntax::parse;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &str| {
    parse(data);
});
