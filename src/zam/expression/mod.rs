pub mod group;
mod number;
pub mod term;
mod text;

use std::fmt::Display;

use group::GroupValue;
use term::Term;

use crate::{
    misc::Bypass,
    parser::{
        misc::ValidID,
        span::{Span, ToSpan},
    },
};

use super::{fields::FieldValue, typ::Type, Parser};

#[derive(Debug, Clone, Default)]
pub struct Expression {
    pub data: Vec<Span<Term>>,
    pub typ: Type,
}

impl From<Vec<Span<Term>>> for Expression {
    fn from(value: Vec<Span<Term>>) -> Self {
        Self {
            data: value,
            typ: Type::default(),
        }
    }
}

impl FieldValue for Expression {
    fn field_value(src: &mut Parser) -> Option<Self> {
        Some(src.exp(',', false)?.0)
    }
}

impl GroupValue for Expression {
    fn group_value(src: &mut Parser) -> Option<Option<Self>> {
        let Some((exp, used)) = src.exp(',', false) else {
            return Some(None);
        };
        let empty = exp.data.is_empty();

        if used {
            src.idx += 1
        }

        if empty {
            return None;
        }

        Some(Some(exp))
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for v in &self.data {
            write!(f, "{v} ")?
        }

        Ok(())
    }
}

const OP: [(char, Term, &[(char, Term)]); 8] = {
    let mut i = 1;
    let mut ops: [(char, Term, &[(char, Term)]); 8] = [
        ('!', Term::Neg, &[('=', Term::Nq)]),
        ('+', Term::Add, &[('=', Term::AddAssign)]),
        ('-', Term::Sub, &[]),
        ('.', Term::Access(false), &[('.', Term::Rng)]),
        ('/', Term::Div, &[]),
        ('<', Term::Gt, &[('<', Term::Shl), ('=', Term::Le)]),
        ('>', Term::Lt, &[('<', Term::Shr), ('=', Term::Ge)]),
        ('=', Term::Assign, &[('=', Term::Eq)]),
    ];

    while i < ops.len() {
        let mut j = i;

        while j > 0 && ops[j - 1].0 > ops[j].0 {
            ops.swap(j - 1, j);
            j -= 1;
        }

        i += 1;
    }

    ops
};

impl Parser {
    pub fn exp(&mut self, de: char, required: bool) -> Option<(Expression, bool)> {
        let mut exp: Vec<Span<_>> = Vec::new();
        let mut end = false;
        let last = match self.de.back() {
            Some(n) => n - 1,
            _ => 0,
        };

        while let Some(c) = self._peek() {
            end = c == de;

            if end || self.idx == last {
                break;
            }

            if c.is_ascii_whitespace() {
                self._next();
                continue;
            }

            let start = self.idx + 1;
            let tmp = if let Some(v) = self.stm_gen()? {
                if match exp.last() {
                    Some(v) => !matches!(v.data, Term::Identifier(_)),
                    _ => true,
                } {
                    self.err("expected identifier before generic parameter")?
                }

                v
            } else if c == '{' {
                match exp.last() {
                    Some(Span {
                        data: Term::Identifier(_),
                        ..
                    }) => {
                        self.idx += 1;
                        Term::Struct(self.fields('}')?)
                    }
                    _ => Term::Block(self.block(false)?),
                }
            } else if c == '(' {
                let mut tmp = self.group()?;
                match tmp.len() {
                    0 => Term::None,
                    1 => Term::Group(tmp.pop().unwrap()),
                    _ => Term::Tuple(tmp),
                }
            } else if c.is_ascii_digit() || c == '-' && exp.is_empty() {
                self.num()?
            } else if c == 'a' && self.peek_more() == 's' {
                self.idx += 2;
                Term::As(self.typ()?)
            } else if match c {
                'b' | 'r' if matches!(self.peek_more(), '\'' | '"') => true,
                '\'' | '"' => true,
                _ => false,
            } {
                self.text()?
            } else if c.is_id() {
                Term::Identifier(self.identifier(true)?)
            } else {
                self._next();

                'one: {
                    if let Ok(i) = OP.binary_search_by_key(&c, |v| v.0) {
                        let v = &OP[i];

                        if let Ok(j) = v.2.binary_search_by_key(&self.peek(), |v| v.0) {
                            self._next();
                            break 'one v.2[j].1.clone();
                        }

                        break 'one v.1.clone();
                    }

                    self.rng.fill(self.idx);
                    self.err_op(false, &[de.to_string().as_str(), "<operator>"])?
                }
            };

            exp.push(tmp.span([start, self.idx]));
        }

        if required && exp.is_empty() {
            self.err_op(true, &["<expression>"])?
        }

        let mut order = [2, 0];
        let mut index = [0; 3];
        let exp_mut = exp.bypass();
        let mut iter = exp.iter().enumerate();

        loop {
            let one = iter.next();

            if let Some((n, v)) = one {
                index[2] = n;
                let tmp = match **v {
                    Term::Div => 2,
                    Term::Add | Term::Sub => 1,
                    _ => continue,
                };

                if order[0] != order[1] {
                    if order[0] != tmp {
                        index[1] = n + 1
                    }
                    order[1] = tmp;
                    continue;
                }
            }

            // in case of emergency try removing this
            if order[0] != order[1] {
                break;
            }

            loop {
                for n in index[0]..index[1] {
                    exp_mut.swap(n, index[1]);
                }

                index[0] += 1;

                if index[1] == index[2] {
                    for n in index[0]..index[2] {
                        exp_mut.swap(n, index[2])
                    }

                    break;
                }

                index[1] += 1;
            }

            if one.is_none() {
                break;
            }
        }

        Some((
            Expression {
                data: exp,
                typ: Type::default(),
            },
            end,
        ))
    }
}
