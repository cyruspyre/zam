use std::{
    borrow::Borrow,
    cmp::Ordering,
    fmt::{self, Debug, Display, Formatter},
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref},
    parser::{
        misc::CharExt,
        span::{Span, ToSpan},
        Parser,
    },
    zam::{expression::misc::Range, misc::display, path::ZamPath},
};

#[derive(Default, Clone, Eq)]
pub struct Identifier(pub(super) Vec<Span<String>>);

impl Parser {
    pub fn identifier(&mut self, required: bool, qualifiable: bool) -> Option<Identifier> {
        let mut buf = Vec::new();
        let mut pnt = Vec::new();
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

            if tmp == "self" {
                pnt.push((log.rng, Point::Error, ""))
            } else {
                buf.push(tmp.span(log.rng))
            }

            let idx = self.idx;
            let qualified = buf.len() > 1;

            if matches!(self.next_if(&["::"]), Ok(_) if self.skip_whitespace().is_id()) {
                continue;
            }

            if qualified && !pnt.is_empty() {
                let msg = "`self` as an identifier cannot be qualified";
                log(&mut pnt, Log::Error, msg, "");
                return None;
            }

            self.idx = idx;
            log.rng = buf.rng();

            if qualified && !qualifiable {
                log.err("expected non-qualified identifier")?
            }

            return Some(Identifier(buf));
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

    pub fn qualify(&self, base: &Self) -> Self {
        let mut tmp = base.to_vec();
        tmp.extend_from_slice(&self.0);
        Self(tmp)
    }

    pub fn relative(&self, base: &ZamPath) -> Self {
        Self(self[base.len()..].to_vec())
    }
}

impl Range for Identifier {
    fn rng(&self) -> [usize; 2] {
        self.0.rng()
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

impl PartialOrd for Identifier {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        for i in 0..self.len().min(other.len()) {
            let cmp = self[i].cmp(&other[i]);

            if cmp.is_ne() {
                return Some(cmp);
            }
        }

        Some(self.len().cmp(&other.len()))
    }
}

impl Ord for Identifier {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
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
