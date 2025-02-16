pub mod group;
mod number;
pub mod term;
mod text;

use group::GroupValue;
use term::Term;

use super::{fields::FieldValue, Parser};

pub type Expression = Vec<Term>;

impl FieldValue for Expression {
    fn field_value(src: &mut Parser) -> Self {
        src.exp(',', false).0
    }
}

impl GroupValue for Expression {
    fn group_value(src: &mut Parser) -> Option<Self> {
        let (exp, used) = src.exp(',', false);
        let empty = exp.is_empty();

        if used {
            src.idx += 1
        }

        if empty {
            return None;
        }

        Some(exp)
    }
}

// manually sort the elements as per ascii ascending order
const OP: [(char, Term, &[(char, Term)]); 8] = [
    ('!', Term::Neg, &[('=', Term::Nq)]),
    ('+', Term::Add, &[('=', Term::AddAssign)]),
    ('-', Term::Sub, &[]),
    ('.', Term::Access(false), &[('.', Term::Rng)]),
    ('/', Term::Div, &[]),
    ('<', Term::Gt, &[('<', Term::Shl), ('=', Term::Le)]),
    ('>', Term::Lt, &[('<', Term::Shr), ('=', Term::Ge)]),
    ('=', Term::Assign, &[('=', Term::Eq)]),
];

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

impl Parser {
    pub fn exp(&mut self, de: char, required: bool) -> (Expression, bool) {
        let mut exp = Vec::new();
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

            let tmp = if c == '{' {
                match exp.last() {
                    Some(Term::Identifier(_)) => {
                        self.idx += 1;
                        Term::Struct(self.fields('}'))
                    }
                    _ => Term::Block(self.block(false)),
                }
            } else if c == '(' {
                let mut tmp = self.group();

                match tmp.len() {
                    0 => Term::Void,
                    1 => Term::Group(tmp.pop().unwrap()),
                    _ => Term::Tuple(tmp),
                }
            } else if c.is_ascii_digit() || c == '-' && exp.is_empty() {
                self.num()
            } else if c == 'a' && self.peek_more() == 's' {
                self.idx += 2;
                Term::As(self.identifier(false))
            } else if match c {
                'b' | 'r' if matches!(self.peek_more(), '\'' | '"') => true,
                '\'' | '"' => true,
                _ => false,
            } {
                self.text()
            } else if c == '_' || c.is_ascii_alphabetic() {
                Term::Identifier(self.identifier(false))
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
