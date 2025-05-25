use crate::{
    parser::{
        log::{Log, Point},
        span::Span,
    },
    zam::{expression::term::AssignKind, Entity},
};

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

        self.rng.fill(self.idx + 1);

        let [typ, de] = match self.next() {
            c if c.is_ascii_alphabetic() => [c, self.next()],
            c => [' ', c],
        };

        self.ensure_closed(de)?;

        if !matches!(typ, 'b' | 'r' | ' ') {
            self.err("unknown prefix")?
        }

        let byte = typ == 'b';
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
                    self.ignore = true;
                    let tmp = self.ensure_closed('}').is_none();
                    self.ignore = false;

                    if tmp {
                        self.log(
                            &mut [(self.rng, Point::Info, "starting here")],
                            Log::Error,
                            "unclosed string interpolation",
                            "if you meant to use `{`, escape it using `\\{`",
                        );
                        return None;
                    }

                    buf.push(WTF::Exp(self.exp(['}'], true)?.0));
                    self.de.pop_back();
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

        self.rng[1] = self.idx;
        self.de.pop_back();

        if de == '"' {
            if buf.len() == 1 {
                return match buf.pop().unwrap() {
                    WTF::Exp(v) => Some(Term::Group(Expression::from(arr![
                        flatten(v),
                        Term::Access,
                        "to_string".into(),
                        Term::Tuple(Vec::new()),
                    ]))),
                    WTF::Buf(data) => Some(Term::String { data, byte }),
                };
            }

            let mut stm = vec![Statement::Variable {
                id: "0".into(),
                data: Entity::Variable {
                    exp: Expression::from(arr![
                        ["String", "with_capacity"].into(),
                        Term::Tuple(vec![Expression::from(arr![Term::Integer {
                            val: size,
                            bit: 64,
                            neg: false,
                            sign: false,
                        }])]),
                    ]),
                    cte: false,
                    done: false,
                },
            }];

            for v in buf {
                let mut exp = Expression::from(arr!["0".into(), Term::Assign(AssignKind::Add)]);
                let tmp: &[Span<Term>] = match v {
                    WTF::Buf(data) => &self.lol([Term::String { data, byte }]),
                    WTF::Exp(v) => &self.lol([
                        flatten(v),
                        Term::Access,
                        "to_string".into(),
                        Term::Tuple(Vec::new()),
                    ]),
                };

                exp.data.extend_from_slice(tmp);
                stm.push(Statement::Expression(exp));
            }

            return Some(Term::Block(Block {
                stm,
                ..Default::default()
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
                            sign: false,
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
