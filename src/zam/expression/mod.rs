mod array;
mod conditional;
pub mod group;
pub mod misc;
mod number;
pub mod term;
mod text;

use std::fmt::{Debug, Display};

use group::GroupValue;
use misc::Range;
use term::{AssignKind, Term};

use crate::{
    log::{Log, Point},
    misc::Bypass,
    parser::{
        misc::CharExt,
        span::{Span, ToSpan},
    },
};

use super::{Parser, block::BlockType, fields::FieldValue, typ::Type};

#[derive(Clone, Default, PartialEq)]
pub struct Expression {
    pub data: Vec<Span<Term>>,
    pub typ: Type,
}

const OP: [(char, Term, &[(char, Term)]); 12] = {
    let mut i = 1;
    let mut ops: [(char, Term, &[(char, Term)]); 12] = [
        ('!', Term::Neg, &[('=', Term::Nq)]),
        ('+', Term::Add, &[]),
        ('-', Term::Sub, &[]),
        ('.', Term::Access, &[('.', Term::Rng)]),
        ('*', Term::Mul, &[]),
        ('/', Term::Div, &[]),
        ('%', Term::Mod, &[]),
        ('|', Term::BitOr, &[('|', Term::Or)]),
        ('&', Term::BitAnd, &[('&', Term::And)]),
        ('<', Term::Gt, &[('<', Term::Shl), ('=', Term::Le)]),
        ('>', Term::Lt, &[('<', Term::Shr), ('=', Term::Ge)]),
        ('=', Term::Assign(AssignKind::Normal), &[('=', Term::Eq)]),
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
    pub fn exp<const N: usize>(
        &mut self,
        de: [char; N],
        required: bool,
    ) -> Option<(Expression, char)> {
        let mut exp: Vec<Span<_>> = Vec::new();
        let mut end = '\0';
        let mut ass = true; // assignable...
        let mut was_op = true;
        let log = self.log.bypass();
        let last = match self.de.front() {
            Some(n) => n - 1,
            _ => 0,
        };

        while let Some(c) = self._peek() {
            if de.contains(&c) {
                end = c;
            }

            if end != '\0' || self.idx == last {
                break;
            }

            if c.is_ascii_whitespace() {
                self._next();
                continue;
            }

            let start = self.idx + 1;
            let mut is_op = false;
            let term = if let Some(v) = self.stm_gen()? {
                if match exp.last() {
                    Some(v) => !matches!(v.data, Term::Identifier(_)),
                    _ => true,
                } {
                    log.err("expected identifier before generic parameter")?
                }

                v
            } else if c == '='
                && let Some(v) = exp.last()
                && v.rng[0] == self.idx
            {
                is_op = true;
                self.idx += 1;

                let kind = match v.data {
                    Term::Add => AssignKind::Add,
                    Term::Sub => AssignKind::Sub,
                    Term::Mul => AssignKind::Mul,
                    Term::Div => AssignKind::Div,
                    _ => AssignKind::Normal,
                };

                if !matches!(kind, AssignKind::Normal) {
                    was_op ^= is_op;
                    ass = true;
                    exp.pop();
                }

                if !ass {
                    log(
                        &mut [
                            (exp.rng(), Point::Info, "cannot assign to this expression"),
                            ([self.idx; 2], Point::Error, ""),
                        ],
                        Log::Error,
                        "invalid assignment operation",
                        "",
                    );
                    return None;
                }

                Term::Assign(kind)
            } else if c == '(' {
                let mut tmp = self.group()?;
                match tmp.len() {
                    0 => Term::None,
                    1 => Term::Group(tmp.pop().unwrap()),
                    _ => Term::Tuple(tmp),
                }
            } else if c == '{' {
                match exp.last() {
                    Some(Span {
                        data: Term::Identifier(_),
                        ..
                    }) => {
                        self.idx += 1;
                        Term::Struct(self.fields('}')?)
                    }
                    _ => Term::Block(self.block(BlockType::Local)?),
                }
            } else if c == '[' {
                self.array()?
            } else if c.is_ascii_digit() || c == '-' && was_op {
                self.num()?
            } else if let Ok(v) = self.next_if(&["as", "if", "true", "false", "return"]) {
                match v.as_str() {
                    "if" => self.conditional()?,
                    "as" => Term::As(self.typ()?),
                    "return" => Term::Return(self.exp([';'], false)?.0),
                    _ => Term::Bool(v.len() == 4), // `true` has 4 chars
                }
            } else if c.is_quote() || c.is_id() && self.peek_more().is_quote() {
                self.text()?
            } else if c.is_id() {
                Term::Identifier(self.identifier(true, true)?)
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

                        break 'one match v.1.clone() {
                            Term::BitAnd if was_op => Term::Ref,
                            Term::Mul if was_op => Term::Deref,
                            v => v,
                        };
                    }

                    log.rng.fill(self.idx);
                    let mut op = Vec::with_capacity(de.len() + 1);

                    for c in de {
                        op.push(c.to_string());
                    }

                    op.push("<operator>".into());

                    log.err_op(false, &op)?
                }
            };
            let special = !matches!(
                term,
                Term::Ref | Term::Deref | Term::Neg | Term::As(_) | Term::Struct(_)
            );

            log.rng = [start, self.idx];
            is_op |= term == Term::Sub;
            ass &= matches!(term, Term::Deref | Term::Identifier(_));

            if special && is_op == was_op {
                log.err_op(
                    false,
                    &[match was_op {
                        true => "<term>",
                        _ => "<operator>",
                    }],
                )?;
            }

            was_op = is_op;
            exp.push(term.span(log.rng));
        }

        if exp.is_empty() {
            if required {
                log.err_op(true, &["<expression>"])?
            }

            return Some((
                Expression {
                    data: exp,
                    ..Default::default()
                },
                end,
            ));
        }

        if was_op {
            log.err_op(true, &["<term>"])?
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
                    Term::Mul | Term::Div | Term::Eq | Term::Nq => 2,
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
                ..Default::default()
            },
            end,
        ))
    }
}

impl Expression {
    pub fn new<const N: usize>(terms: [Term; N], rng: [usize; 2]) -> Self {
        Self {
            data: terms.map(|v| v.span(rng)).to_vec(),
            typ: Type::default(),
        }
    }
}

impl FieldValue for Expression {
    fn field_value(src: &mut Parser) -> Option<Self> {
        Some(src.exp([','], true)?.0)
    }
}

impl GroupValue for Expression {
    fn group_value(src: &mut Parser) -> Option<Option<Self>> {
        let Some((exp, de)) = src.exp([','], false) else {
            return Some(None);
        };
        let empty = exp.data.is_empty();

        if de != '\0' {
            src.idx += 1
        }

        if empty {
            return None;
        }

        Some(Some(exp))
    }
}

impl From<Vec<Span<Term>>> for Expression {
    fn from(value: Vec<Span<Term>>) -> Self {
        Self {
            data: value,
            ..Default::default()
        }
    }
}

impl Debug for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            return f.write_str(&self.to_string());
        }

        f.debug_struct("Expression")
            .field("data", &self.data)
            .field("typ", &self.typ)
            .finish()
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
