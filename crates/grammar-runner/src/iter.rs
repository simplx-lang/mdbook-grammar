use grammar_syntax::SyntaxNode;
use mdbook::book::{Book, Chapter};
use mdbook::BookItem;
use std::mem::transmute;

pub trait RecursiveIterable {
    type Item;

    fn recur_iter(&self) -> impl Iterator<Item=&Self::Item>;

    fn recur_iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item>;
}

pub trait Tree {
    fn children(&self) -> Vec<&Self>;
    fn children_mut(&mut self) -> Vec<&mut Self>;
}

struct TreeIter<'s, T: Tree> {
    stack: Vec<&'s T>,
}

impl<'s, T: Tree> TreeIter<'s, T> {
    fn new(stack: Vec<&'s T>) -> Self {
        Self { stack }
    }

    fn empty() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<'s, T: Tree> Iterator for TreeIter<'s, T> {
    type Item = &'s T;

    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.stack.pop()?;
        self.stack.append(&mut tree.children());
        Some(tree)
    }
}

struct TreeIterMut<'s, T: Tree> {
    stack: Vec<&'s mut T>,
}

impl<'s, T: Tree> TreeIterMut<'s, T> {
    fn new(stack: Vec<&'s mut T>) -> Self {
        Self { stack }
    }

    fn empty() -> Self {
        Self { stack: Vec::new() }
    }
}

impl<'s, T: Tree> Iterator for TreeIterMut<'s, T> {
    type Item = &'s mut T;

    fn next(&mut self) -> Option<Self::Item> {
        let tree = self.stack.pop()?;
        let copy = unsafe { transmute::<&mut T, &mut T>(tree) };
        self.stack.append(&mut copy.children_mut());
        Some(tree)
    }
}

impl<T: Tree> RecursiveIterable for T {
    type Item = T;

    fn recur_iter(&self) -> impl Iterator<Item=&Self::Item> {
        TreeIter::new(vec![self])
    }

    fn recur_iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item> {
        TreeIterMut::new(vec![self])
    }
}

impl<T: Tree> RecursiveIterable for Vec<T> {
    type Item = T;

    fn recur_iter(&self) -> impl Iterator<Item=&Self::Item> {
        TreeIter::new(self.iter().collect())
    }

    fn recur_iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item> {
        TreeIterMut::new(self.iter_mut().collect())
    }
}

impl Tree for Chapter {
    fn children(&self) -> Vec<&Self> {
        self.sub_items
            .iter()
            .filter_map(|item| {
                if let BookItem::Chapter(chapter) = item {
                    Some(chapter)
                } else {
                    None
                }
            })
            .collect()
    }

    fn children_mut(&mut self) -> Vec<&mut Self> {
        self.sub_items
            .iter_mut()
            .filter_map(|item| {
                if let BookItem::Chapter(chapter) = item {
                    Some(chapter)
                } else {
                    None
                }
            })
            .collect()
    }
}

impl RecursiveIterable for BookItem {
    type Item = Chapter;

    fn recur_iter(&self) -> impl Iterator<Item=&Self::Item> {
        if let BookItem::Chapter(chapter) = self {
            TreeIter::new(vec![chapter])
        } else {
            TreeIter::empty()
        }
    }

    fn recur_iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item> {
        if let BookItem::Chapter(chapter) = self {
            TreeIterMut::new(vec![chapter])
        } else {
            TreeIterMut::empty()
        }
    }
}

impl RecursiveIterable for Book {
    type Item = Chapter;

    fn recur_iter(&self) -> impl Iterator<Item=&Self::Item> {
        let chapters = self.sections.iter().filter_map(|item| {
            if let BookItem::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        });

        TreeIter::new(chapters.collect())
    }

    fn recur_iter_mut(&mut self) -> impl Iterator<Item=&mut Self::Item> {
        let chapters = self.sections.iter_mut().filter_map(|item| {
            if let BookItem::Chapter(chapter) = item {
                Some(chapter)
            } else {
                None
            }
        });

        TreeIterMut::new(chapters.collect())
    }
}

impl Tree for SyntaxNode {
    fn children(&self) -> Vec<&Self> {
        self.children().collect()
    }

    fn children_mut(&mut self) -> Vec<&mut Self> {
        unimplemented!()
    }
}
