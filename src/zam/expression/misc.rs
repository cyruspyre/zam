use crate::parser::span::Span;

use super::term::Term;

pub trait Range {
    fn rng(&self) -> [usize; 2];
}

impl Range for Vec<Span<Term>> {
    fn rng(&self) -> [usize; 2] {
        if self.is_empty() {
            return [0; 2];
        }

        [self[0].rng[0], self.last().unwrap().rng[1]]
    }
}
