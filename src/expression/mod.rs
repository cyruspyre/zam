mod number;
mod term;
mod text;

use term::Term;

use crate::source::Source;

const OP: &[(char, Option<Term>, &[(char, Term)])] = &[
    ('=', None, &[('=', Term::Eq)]),
    ('!', Some(Term::Neg), &[('=', Term::Nq)]),
    ('<', Some(Term::Gt), &[('=', Term::Le), ('<', Term::Shl)]),
    ('>', Some(Term::Lt), &[('=', Term::Ge), ('<', Term::Shr)]),
    ('+', Some(Term::Add), &[]),
    ('-', Some(Term::Sub), &[]),
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
    pub fn exp(&mut self, de: char) -> (Expression, bool) {
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

            let tmp = if c.is_ascii_digit() || c == '-' {
                self.num()
            } else if c == 'a' && self.peek_more() == 's' {
                self.idx += 2;
                Term::As(self.identifier())
            } else if c == '_' || c.is_ascii_alphabetic() {
                Term::Identifier(self.identifier())
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
                            break 'one v.clone();
                        }
                    }

                    self.rng.fill(0);
                    self.err_op(false, &[de.to_string().as_str(), "<operator>"])
                }
            };

            exp.push(tmp);
        }

        println!("{:?}", exp);
        println!("{}", exp.to_string());

        (exp, end)
    }
}
