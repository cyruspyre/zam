use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref},
    parser::{
        span::{Span, ToSpan},
        Parser,
    },
    zam::{expression::misc::Range, misc::display, path::ZamPath},
};

#[derive(Default, Clone, Eq)]
pub struct Identifier(pub(super) Vec<Span<String>>);

impl Identifier {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn leaf_name(&self) -> &Span<String> {
        self.0.last().unwrap()
    }

    pub fn is_qualified(&self) -> bool {
        self.0.len() > 1
    }

    pub fn qualify<T: Borrow<String>>(base: &[T]) -> Self {
        todo!()
    }

    pub fn relative(&self, base: &ZamPath) -> Self {
        Self(self[base.len()..].to_vec())
    }
}

impl Deref for Identifier {
    type Target = Vec<Span<String>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Identifier {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Borrow<Span<String>> for Identifier {
    fn borrow(&self) -> &Span<String> {
        self.leaf_name()
    }
}

impl Borrow<Span<String>> for Ref<Identifier> {
    fn borrow(&self) -> &Span<String> {
        self.leaf_name()
    }
}

impl PartialEq for Identifier {
    fn eq(&self, other: &Self) -> bool {
        self.leaf_name() == other.leaf_name()
    }
}

impl Hash for Identifier {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.leaf_name().hash(state);
    }
}

impl Range for Identifier {
    fn rng(&self) -> [usize; 2] {
        self.0.rng()
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self::from([value])
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self::from([value])
    }
}

impl<S: AsRef<str>, const N: usize> From<[S; N]> for Identifier {
    fn from(value: [S; N]) -> Self {
        Self(value.map(|v| v.as_ref().to_string().span([0; 2])).to_vec())
    }
}

impl Debug for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        f.write_str(&format!("\"{self}\""))
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        display(&self.0, f)
    }
}

impl Parser {
    pub fn identifier(&mut self, required: bool, qualifiable: bool) -> Option<Identifier> {
        let mut buf = Vec::new();
        let non_qualifiable = !qualifiable;
        let log = self.log.bypass();
        let idx = self.idx;
        let msg = loop {
            let tmp = self.word();

            if required && tmp.is_empty() {
                break "<identifier>";
            }

            if tmp.chars().next().is_some_and(|c| c.is_ascii_digit()) {
                break "cannot start with a number";
            }

            if matches!(
                tmp.as_str(),
                "fn" | "if"
                    | "in"
                    | "cte"
                    | "let"
                    | "pub"
                    | "use"
                    | "for"
                    | "else"
                    | "loop"
                    | "enum"
                    | "true"
                    | "false"
                    | "while"
                    | "struct"
                    | "extern"
            ) {
                break "cannot be a keyword";
            }

            buf.push(tmp.span(log.rng));

            if non_qualifiable || self.next_if(&["::"]).is_err() {
                log.rng = buf.rng();
                return Some(Identifier(buf));
            }
        };

        match msg.len() {
            0 => {}
            12 => {
                self.skip_whitespace();

                let after = if matches!(self.de.front(), Some(v) if self.idx == v - 1) {
                    true
                } else {
                    self.id_or_symbol();
                    self.de_rng();

                    false
                };

                log.err_op(after, &[msg])?
            }
            _ => log.bypass()(
                &mut [(log.rng, Point::Error, msg)],
                Log::Error,
                "expected `<identifier>`",
                "",
            ),
        }

        if log.ignore {
            self.idx = idx
        }

        None
    }
}
