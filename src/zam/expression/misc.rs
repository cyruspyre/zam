use crate::parser::span::Span;

pub trait Range {
    fn rng(&self) -> [usize; 2];
}

impl<T> Range for Vec<Span<T>> {
    fn rng(&self) -> [usize; 2] {
        if self.is_empty() {
            return [0; 2];
        }

        [self[0].rng[0], self.last().unwrap().rng[1]]
    }
}
