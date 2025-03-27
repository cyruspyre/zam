pub mod generic;
pub mod kind;
mod r#trait;

use kind::TypeKind;

use crate::parser::{misc::ValidID, span::Span};

use super::{expression::group::GroupValue, fields::FieldValue, Parser};

#[derive(Debug, Clone, Default)]
pub struct Type {
    pub kind: Span<TypeKind>,
    pub sub: Vec<Type>,
    pub ptr: usize,
    pub raw: bool,
    pub null: usize,
}

impl FieldValue for Type {
    fn field_value(src: &mut Parser) -> Option<Self> {
        src.typ()
    }
}

impl GroupValue for Type {
    fn group_value(src: &mut Parser) -> Option<Option<Self>> {
        match src.skip_whitespace() {
            ')' => None,
            _ => Some(src.typ()),
        }
    }
}

impl Parser {
    pub fn typ(&mut self) -> Option<Type> {
        let mut ptr = [0; 2];
        let mut tmp = true;

        loop {
            let n = match self.next_char_if(&['*', '&']) {
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
            self.err("cannot mix pointers and reference")?
        }

        let mut fun = self.next_if(&["fn"]).ctx;

        if fun && self.peek().is_id() {
            self.idx -= 2;
            fun = false;
        }

        let tuple = self.skip_whitespace() == '(';
        let name = if !fun && !tuple {
            self.identifier(true)
        } else {
            None
        };
        let raw = ptr[0] > 0;
        let idx = self.rng[0];
        let mut sub = Vec::new();

        if !tuple && self.might('<') {
            self.ensure_closed('>')?;

            loop {
                if self.might('>') {
                    break;
                }

                self.rng.fill(0);
                sub.push(self.typ()?);

                if self.might('>') {
                    break;
                }

                self.expect_char(&[',']);
            }

            self.de.pop_back();
        }

        let data = if fun {
            TypeKind::Fn {
                arg: self.group()?,
                ret: {
                    self.expect(&["->"])?;
                    Box::new(self.typ()?)
                },
            }
        } else if tuple {
            TypeKind::Tuple(self.group()?)
        } else {
            TypeKind::ID(name?.data)
        };

        let mut null = 0;

        while self.might('?') {
            null += 1
        }

        if raw && null != 0 {
            self.err("cannot use nullable indicator in pointers");
        }

        Some(Type {
            kind: Span {
                rng: [idx, self.idx],
                data,
            },
            sub,
            ptr: ptr[!raw as usize],
            raw,
            null,
        })
    }
}
