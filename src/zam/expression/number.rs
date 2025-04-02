use hexf_parse::parse_hexf64;

use super::{Parser, Term};

impl Parser {
    pub fn num(&mut self) -> Option<Term> {
        let mut buf = [const { String::new() }; 2];
        let mut suf = '\0';
        let mut dot = false;
        let neg = self.might('-');

        if neg && !self.skip_whitespace().is_ascii_digit() {
            return Some(Term::Sub);
        }

        self.rng.fill(self.idx);

        let rad = if self.might('0') {
            match self.next() {
                'b' | 'B' => 2,
                'o' | 'O' => 8,
                'x' | 'X' => 16,
                _ => {
                    buf[0].push('0');
                    self.idx -= 1;
                    self.rng[0] -= neg as usize;
                    10
                }
            }
        } else {
            10
        };

        while let Some(mut c) = self._next() {
            if c == '_' {
                continue;
            }

            if c == '.' && {
                let c = self.peek();

                c != '.' && !c.is_ascii_alphabetic()
            } {
                buf[0].push('.');
                dot = true;
                c = self.next();
            }

            match c {
                '+' | '-' | 'p' if rad == 16 => buf[0].push(c),
                'i' | 'u' | 'f'
                    if buf[1].is_empty()
                        && (rad != 16 || rad == 16 && suf == 'f' || rad == 16 && c != 'f') =>
                {
                    buf.swap(0, 1);
                    suf = c
                }
                _ if c.is_ascii_digit() || rad == 16 && c.is_ascii_hexdigit() => buf[0].push(c),
                _ => {
                    self.idx -= 1;

                    if !c.is_ascii_alphabetic() {
                        break;
                    }

                    let tmp = self.rng[0];

                    if self.word() == "size" && suf != 'f' {
                        buf[0] += "size"
                    }

                    self.rng[0] = tmp;
                    break;
                }
            }
        }

        self.rng[1] = self.idx;

        if buf[1].len() != 0 {
            buf.swap(0, 1)
        }

        let bit = match buf[1].as_str() {
            "" if suf == '\0' => 0,
            "size" => u32::MAX,
            v => 'a: {
                let tmp = v.parse().unwrap_or_default();

                if neg && suf == 'u' {
                    self.err("unsigned integer cannot be negative")?
                }

                self.err(format!(
                    "invalid suffix. expected {suf}{{{}}}",
                    match suf {
                        'f' if !matches!(tmp, 32 | 64) => "32|64",
                        _ if !matches!(tmp, 1..=64) => "1..=64|size",
                        _ => break 'a tmp,
                    },
                ))?
            }
        };

        if dot {
            suf = 'f'
        }

        match suf {
            'f' => {
                if let Some(val) = match rad {
                    10 => buf[0].parse().ok(),
                    16 => parse_hexf64(&format!("0x{}", buf[0]), false).ok(),
                    _ => self.err("only decimal and hexadecimal are allowed for float literals")?,
                } {
                    return Some(Term::Float { val, bit });
                }
            }
            _ => 'tmp: {
                let mut val = match u64::from_str_radix(&buf[0], rad) {
                    Ok(n) => n,
                    _ => break 'tmp,
                };

                if neg {
                    val = val.wrapping_neg()
                }

                return Some(Term::Integer {
                    sign: suf == 'i',
                    val,
                    bit,
                    neg,
                });
            }
        }

        self.err(format!(
            "invalid {} {} literal",
            match rad {
                2 => "binary",
                8 => "octal",
                10 => "decimal",
                _ => "hexadecimal",
            },
            match suf {
                'f' => "float",
                _ => "integer",
            }
        ))?
    }
}
