use crate::{external::prepare::Prepare, misc::ValidID, source::Source};

use super::External;

macro_rules! cleanup {
    ($self:ident) => {
        if $self.expect_char(&[',', '}']) == '}' || $self.might('}') {
            break;
        }
    };
}

pub type Libs = Vec<Lib>;
#[derive(Debug)]
pub struct Lib {
    pub name: String,
    pub rng: [usize; 2],
    pub ids: Vec<(String, [usize; 2])>,
}

impl Source {
    pub fn ext_use(&mut self, ext: &mut Vec<External>) {
        let mut libs = Vec::new();

        match self.skip_whitespace() {
            '{' => {
                self.next();

                loop {
                    if libs.len() != 0 && !matches!(self.skip_whitespace(), '"' | '}') {
                        self.er(&["\"", "}"]);
                    }

                    libs.push(self.lib());
                    cleanup!(self)
                }
            }
            '"' => libs.push(self.lib()),
            _ => self.err_op(false, &['"', '{']),
        }

        self.expect_char(&[';']);
        println!("{:?}", libs);
        libs.prepare(self, ext);
        // prepare(libs);
    }

    fn er(&mut self, op: &[&str]) {
        // self.after = self._next().is_none();
        // if !self.after {
        //     self.rng = [self.idx; 2];
        // }
        self.err_op(false, op)
    }

    fn lib(&mut self) -> Lib {
        let mut ids = Vec::new();
        let name = self.enclosed('"');
        let rng = self.rng;
        self.expect_char(&['{']);

        loop {
            if self.might('*') {
                let idx = self.idx;
                let op: &[char] = match self.might(',') {
                    true => {
                        if ids.len() != 0 || self.skip_whitespace().is_id() {
                            self.rng = [idx; 2];
                            self.err("cannot mix wildcard and individual identifiers")
                        }

                        &['}']
                    }
                    _ => &[',', '}'],
                };

                self.expect_char(op);
                return Lib {
                    name,
                    rng,
                    ids: vec![(String::new(), [idx; 2])],
                };
            }

            let tmp = self.word();

            if tmp.is_empty() {
                let op = &mut ["}", "<identifier>"];

                if ids.is_empty() {
                    op[0] = "*"
                }

                self.er(op);
            }

            ids.push((tmp, self.rng));
            cleanup!(self);
        }

        Lib { name, rng, ids }
    }
}
