use hexf_parse::parse_hexf64;

use crate::source::Source;

use super::Term;

impl Source {
    pub fn num(&mut self) -> Term {
        let mut buf = [const { String::new() }; 2];
        let mut suf = 'i';
        let neg = self.might('-');

        if neg {
            if !self.skip_whitespace().is_ascii_digit() {
                return Term::Sub;
            }
        }

        self.rng.fill(self.idx + !neg as usize);

        let rad = if self.might('0') {
            match self.next() {
                'b' | 'B' => 2,
                'o' | 'O' => 8,
                'x' | 'X' => 16,
                _ => {
                    buf[0].push('0');
                    self.idx -= 1;
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

            if c == '.' && !self.peek().is_ascii_alphabetic() {
                buf[0].push('.');
                c = self.next();
                suf = 'f';
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
                        // hard coded native size for now
                        buf[0] += "64"
                    } else if buf[1].is_empty() {
                        suf = '\0'
                    }

                    self.rng[0] = tmp;
                    break;
                }
            }
        }

        self.rng[1] = self.idx;

        if buf[1].is_empty() && suf != '\0' {
            buf[1] += "0";
        } else {
            buf.swap(0, 1)
        }

        let bit = buf[1].parse().unwrap_or(65u32);

        'tmp: {
            if bit != 0 {
                let mut msg = "invalid suffix".to_string();

                if suf != '\0' {
                    msg += &format!(
                        ". expected {suf}{{{}}}",
                        match suf {
                            'f' if !matches!(bit, 32 | 64) => "32|64",
                            _ if !matches!(bit, 1..=64) => "1..=64|size",
                            _ => break 'tmp,
                        },
                    )
                }

                self.err(&msg)
            }
        }

        match suf {
            'f' => {
                if let Some(val) = match rad {
                    10 => buf[0].parse().ok(),
                    16 => parse_hexf64(&format!("0x{}", buf[0]), false).ok(),
                    _ => self.err("only decimal and hexadecimal are allowed for float literals"),
                } {
                    return Term::Float { val, bit };
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

                return Term::Integer {
                    val,
                    bit,
                    neg,
                    rng: self.rng,
                    sign: suf == 'i',
                };
            }
        }

        self.err(&format!(
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
        ))
    }
}
