mod block;
mod error;
mod expression;
mod external;
mod fields;
mod function;
mod misc;
mod source;
mod statement;
mod r#struct;
mod typ;

use std::{str::Chars, vec::IntoIter};

use clang::{Clang, EntityKind, Index};
use source::Source;
use unicode_segmentation::Graphemes;

fn main() {
    let src = Source::new("main.z").parse();

    // println!("{:#?}", src.fun[0].block.stm)
}
