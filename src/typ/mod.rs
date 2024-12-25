mod generic;
mod r#trait;

pub use generic::*;
pub use r#trait::*;

use crate::source::Source;

#[derive(Debug, Default, Clone)]
#[allow(unused)]
pub struct Type {
    pub name: String,
    pub sub: Vec<Type>,
    pub ptr: usize,
    pub raw: bool,
    pub null: usize,
}

impl Source {
    pub fn typ(&mut self) -> Type {
        let mut ptr = [0; 2];
        let mut tmp = true;

        loop {
            let n = match self.next_if(&['*', '&']) {
                '*' => 0,
                '&' => 1,
                _ => break,
            };

            ptr[n] += 1;
            self.rng[1] = self.idx;

            if tmp {
                self.rng[0] = self.idx;
                tmp = false;
            }
        }

        if ptr[0] > 0 && ptr[1] > 0 {
            self.err("cannot mix pointers and reference")
        }

        let raw = ptr[0] > 0;
        let name = self.identifier();
        let mut null = 0;
        let mut sub = Vec::new();

        if self.might('<') {
            self.ensure_closed('>');

            loop {
                if self.might('>') {
                    break;
                }

                self.rng.fill(0);
                sub.push(self.typ());

                if self.might('>') {
                    break;
                }

                self.expect_char(&[',']);
            }

            self.de.pop();
        }

        while self.might('?') {
            null += 1
        }

        if raw && null != 0 {
            self.err("cannot use nullable indicator in pointers");
        }

        Type {
            name,
            sub,
            ptr: ptr[!raw as usize],
            raw,
            null,
        }
    }
}
