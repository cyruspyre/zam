use crate::{
    log::{Log, Logger, Point},
    misc::Bypass,
    parser::span::Span,
    zam::{Entity, expression::term::AssignKind},
};

use super::{
    super::{Block, statement::Statement},
    Expression, Parser, Term,
};

#[derive(Debug)]
enum Segment {
    Buf(String),
    Exp(Expression),
}

impl Parser {
    pub fn text(&mut self) -> Option<Term> {
        let log = self.log.bypass();

        log.rng.fill(self.idx + 1);

        let [typ, de] = match self.next() {
            c if c.is_ascii_alphabetic() => [c, self.next()],
            c => [' ', c],
        };

        self.ensure_closed(de)?;

        if !matches!(typ, 'b' | 'r' | ' ') {
            log.err("unknown prefix")?
        }

        let byte = typ == 'b';
        let mut buf: Vec<Segment> = Vec::new();
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
                        _ => log.err_rng([self.idx - 1, self.idx], "unknown character escape")?,
                    }
                } else if c == '{' {
                    let Logger { rng, ignore, .. } = self.log.bypass();

                    rng.fill(self.idx);
                    *ignore = true;
                    let tmp = self.ensure_closed('}').is_none();
                    *ignore = false;

                    if tmp {
                        log.bypass()(
                            &mut [(log.rng, Point::Info, "starting here")],
                            Log::Error,
                            "unclosed string interpolation",
                            "if you meant to use `{`, escape it using `\\{`",
                        );
                        return None;
                    }

                    buf.push(Segment::Exp(self.exp(['}'], true)?.0));
                    self.de.pop_front();
                    self.idx += 1;
                    continue;
                }
            }

            if !matches!(buf.last(), Some(Segment::Buf(_))) {
                buf.push(Segment::Buf(String::new()))
            }

            if let Some(Segment::Buf(v)) = buf.last_mut() {
                v.push(c);
                size += 1;
            }
        }

        log.rng[1] = self.idx;
        self.de.pop_front();

        if de == '"' {
            if buf.len() == 1 {
                return match buf.pop().unwrap() {
                    Segment::Exp(v) => Some(Term::Group(Expression::new(
                        [
                            flatten(v),
                            Term::Access,
                            "to_string".into(),
                            Term::Tuple(Vec::new()),
                        ],
                        log.rng,
                    ))),
                    Segment::Buf(data) => Some(Term::String { data, byte }),
                };
            }

            let tmp = Expression::new(
                [
                    ["String", "with_capacity"].into(),
                    Term::Tuple(vec![Expression::new(
                        [Term::Integer {
                            val: size,
                            bit: 64,
                            neg: false,
                            sign: false,
                        }],
                        log.rng,
                    )]),
                ],
                log.rng,
            );
            let mut stm = vec![Statement::Variable {
                id: "0".into(),
                data: Entity::Variable {
                    exp: tmp,
                    cte: false,
                    done: false,
                },
            }];

            for v in buf {
                let mut exp = Expression::new(["0".into(), Term::Assign(AssignKind::Add)], log.rng);
                let tmp: &[Span<Term>] = match v {
                    Segment::Buf(data) => &self.lol([Term::String { data, byte }]),
                    Segment::Exp(v) => &self.lol([
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
            Some(Segment::Buf(buf)) if buf.is_empty() => {
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

        log.err(msg)?
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
