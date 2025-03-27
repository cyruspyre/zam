use std::collections::HashMap;

use crate::parser::span::Span;

use super::{
    super::{statement::Statement, Block},
    Expression, Parser, Term,
};

#[derive(Debug)]
enum WTF {
    Buf(String),
    Exp(Expression),
}

impl Parser {
    pub fn text(&mut self) -> Option<Term> {
        macro_rules! arr {
            ($($x:expr),+ $(,)?) => {[$($x),+].map(|v| self.span(v)).to_vec()};
        }

        let [typ, de] = match self.next() {
            c if c.is_ascii_alphabetic() => [c, self.next()],
            c => [' ', c],
        };
        let tmp = self.idx;
        let mut buf: Vec<WTF> = Vec::new();
        let mut size = 0;

        while let Some(mut c) = self._next() {
            if c == de {
                break;
            }

            if typ != 'r' {
                if c == '\\' {
                    c = match self.next() {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        '0' => '\0',
                        '"' => '"',
                        '\'' => '\'',
                        '\\' => '\\',
                        '{' if de == '"' => '{',
                        _ => self.err_rng([self.idx - 1, self.idx], "unknown character escape")?,
                    }
                } else if c == '{' {
                    self.rng.fill(self.idx);
                    buf.push(WTF::Exp(self.exp('}', true)?.0));
                    self.de.pop_front();
                    self.idx += 1;
                    continue;
                }
            }

            if !matches!(buf.last(), Some(WTF::Buf(_))) {
                buf.push(WTF::Buf(String::new()))
            }

            if let Some(WTF::Buf(v)) = buf.last_mut() {
                v.push(c);
                size += 1;
            }
        }

        self.rng = [tmp, self.idx];

        if de == '"' {
            if buf.len() == 1 {
                return match buf.pop().unwrap() {
                    WTF::Exp(v) => Some(Term::Group(Expression::from(arr![
                        flatten(v),
                        Term::Access(false),
                        Term::Identifier(self.span("to_string".into())),
                        Term::Tuple(Vec::new()),
                    ]))),
                    WTF::Buf(data) => Some(Term::String { data, byte: false }),
                };
            }

            let mut stm = vec![Statement::Variable {
                name: self.span("0".into()),
                val: Expression::from(arr![
                    Term::Identifier(self.span("String".into())),
                    Term::Access(true),
                    Term::Identifier(self.span("with_capacity".into())),
                    Term::Tuple(vec![Expression::from(arr![Term::Integer {
                        val: size,
                        bit: 64,
                        neg: false,
                        sign: Some(false),
                    }])]),
                ]),
                cte: false,
            }];

            for v in buf {
                let mut exp = Expression::from(arr![
                    Term::Identifier(self.span("0".into())),
                    Term::AddAssign
                ]);
                let tmp: &[Span<Term>] = match v {
                    WTF::Buf(data) => &self.lol([Term::String { data, byte: false }]),
                    WTF::Exp(v) => &self.lol([
                        flatten(v),
                        Term::Access(false),
                        Term::Identifier(self.span("to_string".into())),
                        Term::Tuple(Vec::new()),
                    ]),
                };

                exp.data.extend_from_slice(tmp);
                stm.push(Statement::Expression(exp));
            }

            return Some(Term::Block(Block {
                dec: HashMap::new(),
                stm,
            }));
        }

        let msg = match buf.pop() {
            Some(WTF::Buf(buf)) if buf.is_empty() => {
                if typ == 'r' {
                    "raw character literal is not allowed"
                } else if buf.chars().skip(1).next().is_some() {
                    "character literal may contain only one codepoint"
                } else if typ == 'b' && buf.len() != 1 {
                    "byte character literal must be ascii"
                } else {
                    return match typ {
                        'b' => Some(Term::Integer {
                            val: buf.as_bytes()[0].into(),
                            bit: 8,
                            neg: false,
                            sign: Some(false),
                        }),
                        _ => Some(Term::Char(buf.chars().next().unwrap())),
                    };
                }
            }
            None => "empty character literal",
            _ => "cannot use interpolation in character literal",
        };

        self.err(msg)?
    }

    fn lol<const N: usize>(&self, v: [Term; N]) -> [Span<Term>; N] {
        v.map(|v| self.span(v))
    }
}

fn flatten(mut v: Expression) -> Term {
    let exp = &mut v.data;

    match exp.len() == 1 {
        true => exp.pop().unwrap().data,
        _ => Term::Group(v),
    }
}
