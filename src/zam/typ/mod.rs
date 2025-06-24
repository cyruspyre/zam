pub mod generic;
pub mod kind;
mod misc;

use std::fmt::{Debug, Display, Formatter, Result};

use kind::TypeKind;
use misc::join;

use crate::{
    misc::Bypass,
    parser::{misc::CharExt, span::Span},
};

use super::{expression::group::GroupValue, fields::FieldValue, Parser};

#[derive(Clone, Default, PartialEq)]
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

impl Parser {
    pub fn typ(&mut self) -> Option<Type> {
        let mut rng = [0; 2];
        let mut ptr = [0; 2];
        let log = self.log.bypass();

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
            log.err_rng(rng, "cannot mix pointers and reference")?
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
            rng[0] = log.rng[0]
        }

        rng[1] = log.rng[1];

        if !tuple && self.might('<') {
            self.ensure_closed('>')?;

            loop {
                if self.might('>') {
                    break;
                }

                log.rng.fill(0);
                sub.push(self.typ()?);

                if self.might('>') {
                    break;
                }

                self.expect_char(&[','])?;
            }

            self.de.pop_front();
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
        log.rng = rng;

        if raw && null != 0 {
            log.err("cannot use nullable indicator with raw pointers")?
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

impl Display for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
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

impl Debug for Type {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        if f.alternate() {
            return Display::fmt(self, f);
        }

        f.debug_struct("Type")
            .field("kind", &self.kind)
            .field("sub", &self.sub)
            .field("ptr", &self.ptr)
            .field("raw", &self.raw)
            .field("null", &self.null)
            .finish()
    }
}
