use crate::source::Source;

use super::Term;

type Expr<'a> = &'a mut Vec<Term>;

impl Source {
    pub fn txt(&mut self, exp: Expr) {
        let [typ, de] = match self.next() {
            c if c.is_ascii_alphabetic() => [c, self.next()],
            c => [' ', c],
        };
        let mut buf = String::new();

        loop {
            let mut c = self.next();

            if c == de {
                break;
            }

            if typ != 'r' && c == '\\' {
                c = match self.next() {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '0' => '\0',
                    '"' => '"',
                    '\'' => '\'',
                    '\\' => '\\',
                    _ => self.err("unknown character escape"),
                };
            }

            buf.push(c);
        }

        match de {
            '"' => exp.push(Term::String {
                data: buf,
                byte: typ == 'b',
            }),
            _ => {
                let msg = if typ == 'r' {
                    "raw character literal is invalid"
                } else if buf.is_empty() {
                    "empty character literal"
                } else if buf.chars().skip(1).next().is_some() {
                    "character literal may only contain one codepoint"
                } else if typ == 'b' && buf.len() != 1 {
                    "byte character literal must be ascii"
                } else {
                    exp.push(match typ {
                        'b' => Term::Integer {
                            val: buf.as_bytes()[0].into(),
                            bit: 8,
                            neg: false,
                            rng: [0; 2],
                            sign: false,
                        },
                        _ => Term::Char(buf.chars().next().unwrap()),
                    });
                    return;
                };

                if typ != ' ' {
                    buf += " "
                }
                buf += "  ";
                // self.word = buf;
                self.err(msg);
            }
        }
    }
}
