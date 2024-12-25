use hexf_parse::parse_hexf64;

use crate::{misc::ValidID, source::Source};

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
                'i' | 'u' | 'f' => {
                    buf.swap(0, 1);
                    suf = c
                }
                _ if c.is_ascii_digit() => buf[0].push(c),
                _ => {
                    if c == 's' && self.word() == "ize" && suf != 'f' {
                        todo!("{buf:?}");
                    }
                    self.idx -= 1;
                    break;
                }
            }
        }

        self.rng[1] = self.idx;

        if buf[1].is_empty() {
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
                            _ if !matches!(bit, 1..=64) => "1..=64",
                            _ => break 'tmp,
                        }
                    )
                }

                self.err(&msg)
            }
        }

        println!("{buf:?} {rad} {suf:?} {bit}");

        match suf {
            'f' => todo!(),
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

        println!("{buf:?} {rad} {suf:?} {bit}");

        todo!()
    }
}
