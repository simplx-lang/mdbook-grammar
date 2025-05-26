#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use mdbook::{
    BookItem,
    book::{Book, Chapter},
};
use mdbook_grammar_runner::run;
use std::path::PathBuf;

#[derive(Debug)]
#[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
struct MyBook {
    path: PathBuf,
    content: String,
    children: Vec<MyBook>,
}

impl MyBook {
    fn into_book(self) -> Book {
        let mut book = Book::new();
        book.sections = vec![self.into_item()];
        book
    }

    fn into_item(self) -> BookItem {
        let mut c = Chapter::new("name", self.content, self.path, Vec::new());
        c.sub_items = self
            .children
            .into_iter()
            .map(|child| child.into_item())
            .collect();
        BookItem::Chapter(c)
    }
}

fuzz_target!(|book: MyBook| {
    run(&mut book.into_book(), "/");
});
