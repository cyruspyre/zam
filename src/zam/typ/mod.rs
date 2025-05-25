pub mod generic;
pub mod kind;
mod misc;
mod r#trait;

use std::fmt::Display;

use kind::TypeKind;
use misc::join;

use crate::parser::{misc::CharExt, span::Span};

use super::{expression::group::GroupValue, fields::FieldValue, Parser};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Type {
    /// Span range covers the entire type declaration.
    ///
    /// For example, in `&Vec<u8>`, the range includes the entire type,
    /// not just the base `Vec`
    pub kind: Span<TypeKind>,
    pub sub: Vec<Type>,
    pub ptr: usize,
    pub raw: bool,
    pub null: usize,
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}{}{}",
            if self.raw { "*" } else { "&" }.repeat(self.ptr),
            self.kind,
            if self.sub.is_empty() {
                "".into()
            } else {
                format!("<{}>", join(&self.sub))
            },
            "?".repeat(self.null)
        )
    }
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
        let mut rng = [0; 2];
        let mut ptr = [0; 2];

        loop {
            let n = match self.next_char_if(&['*', '&']) {
                '*' => 0,
                '&' => 1,
                _ => break,
            };

            ptr[n] += 1;

            if rng[0] == 0 {
                rng.fill(self.idx);
            } else {
                rng[1] = self.idx;
            }
        }

        if ptr[0] > 0 && ptr[1] > 0 {
            self.err_rng(rng, "cannot mix pointers and reference")?
        }

        let mut fun = self.might("fn");

        if fun && self.peek().is_id() {
            self.idx -= 2;
            fun = false;
        }

        let tuple = self.skip_whitespace() == '(';
        let name = if !fun && !tuple {
            self.identifier(true, true)
        } else {
            None
        };
        let raw = ptr[0] > 0;
        let mut sub = Vec::new();

        if rng[0] == 0 {
            rng[0] = self.rng[0]
        }

        rng[1] = self.rng[1];

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

                self.expect_char(&[','])?;
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
            TypeKind::ID(name?)
        };

        let mut null = 0;

        while self.might('?') {
            null += 1
        }

        rng[1] = self.idx;
        self.rng = rng;

        if raw && null != 0 {
            self.err("cannot use nullable indicator with raw pointers")?
        }

        Some(Type {
            kind: Span { rng, data },
            sub,
            ptr: ptr[!raw as usize],
            raw,
            null,
        })
    }
}
