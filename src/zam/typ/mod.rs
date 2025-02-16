pub mod generic;
mod r#trait;

use super::{expression::group::GroupValue, fields::FieldValue, Parser};

#[derive(Debug, Default, Clone)]
pub struct Type {
    pub name: String,
    pub sub: Vec<Type>,
    pub ptr: usize,
    pub raw: bool,
    pub null: usize,
}

impl FieldValue for Type {
    fn field_value(src: &mut Parser) -> Self {
        src.typ()
    }
}

impl GroupValue for Type {
    fn group_value(src: &mut Parser) -> Option<Self>
    where
        Self: Sized,
    {
        match src.skip_whitespace() {
            ')' => None,
            _ => Some(src.typ()),
        }
    }
}

impl Parser {
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

        let tuple = self.skip_whitespace() == '(';
        let raw = ptr[0] > 0;
        let mut null = 0;
        let (name, mut sub) = match tuple {
            true => {
                self.rng.fill(self.idx + 1);
                (String::new(), self.group())
            }
            _ => (self.identifier(false), Vec::new()),
        };

        if !tuple && self.might('<') {
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

            self.de.pop_back();
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
