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
        misc::CharExt,
        span::{Span, ToSpan},
    },
};

use super::{fields::FieldValue, typ::Type, Parser};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Expression {
    pub data: Vec<Span<Term>>,
    pub typ: Type,
}

impl Expression {
    pub fn exp_rng(&self) -> [usize; 2] {
        let tmp = &self.data;

        if tmp.is_empty() {
            return [0; 2];
        }

        [tmp[0].rng[0], tmp.last().unwrap().rng[1]]
    }
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
        f.write_str(
            &self
                .data
                .iter()
                .map(|v| v.to_string())
                .collect::<Vec<_>>()
                .join(" "),
        )
    }
}

const OP: [(char, Term, &[(char, Term)]); 10] = {
    let mut i = 1;
    let mut ops: [(char, Term, &[(char, Term)]); 10] = [
        ('!', Term::Neg, &[('=', Term::Nq)]),
        ('+', Term::Add, &[('=', Term::AddAssign)]),
        ('-', Term::Sub, &[]),
        ('.', Term::Access(false), &[('.', Term::Rng)]),
        ('*', Term::Mul, &[]),
        ('/', Term::Div, &[]),
        ('%', Term::Mod, &[]),
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
        let mut was_op = true;
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
            let mut is_op = false;
            let mut special = true;
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
            } else if c.is_ascii_digit() || c == '-' && was_op {
                self.num()?
            } else if self.next_if(&["as"]).is_ok() {
                Term::As(self.typ()?)
            } else if c.is_quote() || c.is_id() && self.peek_more().is_quote() {
                self.text()?
            } else if c.is_id() {
                Term::Identifier(self.identifier(true)?.data)
            } else {
                self._next();
                is_op = true;

                'one: {
                    if let Ok(i) = OP.binary_search_by_key(&c, |v| v.0) {
                        let v = &OP[i];

                        if let Ok(j) = v.2.binary_search_by_key(&self.peek(), |v| v.0) {
                            self._next();
                            break 'one v.2[j].1.clone();
                        }

                        special = false;

                        break 'one match v.1.clone() {
                            Term::Mul if was_op => Term::Deref,
                            v => {
                                special = true;
                                v
                            }
                        };
                    }

                    self.rng.fill(self.idx);
                    self.err_op(false, &[de.to_string().as_str(), "<operator>"])?
                }
            };

            self.rng = [start, self.idx];
            is_op |= tmp == Term::Sub;

            if special && is_op == was_op && exp.len() != 0 {
                let msg = format!(
                    "expected {}",
                    match was_op {
                        true => "a term",
                        _ => "an operator",
                    }
                );

                self.err(msg)?;
            }

            was_op = is_op;
            exp.push(tmp.span(self.rng));
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
