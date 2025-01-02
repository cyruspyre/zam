mod number;
pub mod term;
mod text;

use std::{
    collections::{BinaryHeap, HashSet},
    io::{stdin, Read},
};

use term::Term;

use crate::{source::Source, statement::Statement};

const OP: &[(char, Option<Term>, &[(char, Term)])] = &[
    ('=', Some(Term::Assign), &[('=', Term::Eq)]),
    ('!', Some(Term::Neg), &[('=', Term::Nq)]),
    ('<', Some(Term::Gt), &[('=', Term::Le), ('<', Term::Shl)]),
    ('>', Some(Term::Lt), &[('=', Term::Ge), ('<', Term::Shr)]),
    ('+', Some(Term::Add), &[('=', Term::AddAssign)]),
    ('-', Some(Term::Sub), &[]),
    ('/', Some(Term::Div), &[]),
    ('.', Some(Term::Access(String::new())), &[('.', Term::Rng)]),
];

pub type Expression = Vec<Term>;

pub trait PrettyExp {
    fn to_string(&self) -> String;
}

impl PrettyExp for Vec<Term> {
    fn to_string(&self) -> String {
        self.into_iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

impl Source {
    pub fn exp(&mut self, de: char, required: bool) -> (Expression, bool) {
        let mut exp = Vec::new();
        let mut end = false;
        let last = match self.de.last() {
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

            let tmp = if c.is_ascii_digit() || c == '-' && exp.is_empty() {
                self.num()
            } else if c == 'a' && self.peek_more() == 's' {
                self.idx += 2;
                Term::As(self.identifier(false))
            } else if c == '_' || c.is_ascii_alphabetic() {
                if exp
                    .last()
                    .is_some_and(|v| matches!(v, Term::Access(v) if v.is_empty()))
                {
                    if let Term::Access(v) = exp.last_mut().unwrap() {
                        *v = self.identifier(false)
                    }
                    continue;
                }

                Term::Identifier(self.identifier(false))
            } else {
                self._next();

                'one: {
                    for v in OP {
                        if v.0 != c {
                            continue;
                        }

                        let c = self.peek();

                        for v in v.2 {
                            if v.0 == c {
                                self._next();
                                break 'one v.1.clone();
                            }
                        }

                        if let Some(v) = &v.1 {
                            if let Term::Assign = *v {}
                            break 'one v.clone();
                        }
                    }

                    self.rng.fill(0);
                    self.err_op(false, &[de.to_string().as_str(), "<operator>"])
                }
            };

            exp.push(tmp);
        }

        if required && exp.is_empty() {
            self.err_op(true, &["<expression>"])
        }

        let mut order = [2, 0];
        let mut index = [0; 3];
        let exp_mut: &mut Vec<Term> = unsafe { &mut *(&mut exp as *mut _) };
        let mut iter = exp.iter().enumerate();

        loop {
            let one = iter.next();

            if let Some((n, v)) = one {
                index[2] = n;
                let tmp = match v {
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

        (exp, end)
    }
}
