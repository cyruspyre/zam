use std::fmt::{self, Debug, Display, Formatter};

use crate::parser::{
    log::{Log, Point},
    span::{Span, ToSpan},
    Parser,
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
        let idx = self.idx;
        let non_qualifiable = !qualifiable;
        let mut buf = Vec::new();

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

            buf.push(tmp.span(self.rng));

            if non_qualifiable || self.next_if(&["::"]).is_err() {
                self.rng = buf.rng();
                return Some(Identifier(buf));
            }
        };

        match msg.len() {
            0 => {}
            12 => {
                let rng = self.rng;
                let mut after = self.id_or_symbol().is_none();

                if self.data[self.rng[0] - 1] == '\n' {
                    self.rng = rng;
                    after = true;
                }

                self.err_op(after, &[msg])?
            }
            _ => self.log(
                &mut [(self.rng, Point::Error, msg)],
                Log::Error,
                "expected `<identifier>`",
                "",
            ),
        }

        if self.ignore {
            self.idx = idx
        }

        None
    }
}
