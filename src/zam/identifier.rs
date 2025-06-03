use std::{
    borrow::Borrow,
    fmt::{self, Debug, Display, Formatter},
    ops::{Deref, DerefMut},
};

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref},
    parser::{
        span::{Span, ToSpan},
        Parser,
    },
};

use super::expression::misc::Range;

#[derive(Default, Clone, PartialEq, Eq, Hash)]
pub struct Identifier(Vec<Span<String>>);

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
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&format!("\"{self}\""))
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        let tmp = &self.0;

        for i in 0..tmp.len().checked_sub(1).unwrap_or_default() {
            buf += &tmp[i];
            buf += "::";
        }

        if let Some(v) = tmp.last() {
            buf += v
        }

        f.write_str(&buf)
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
                let rng = log.rng;
                let mut after = self.id_or_symbol().is_none();

                if log.data[log.rng[0] - 1] == '\n' {
                    log.rng = rng;
                    after = true;
                }

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
