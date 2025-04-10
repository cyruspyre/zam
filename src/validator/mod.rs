mod identifier;
mod main_fn;
mod typ;
mod variable;

use std::collections::HashMap;

use crate::{cfg::Config, err, zam::Zam};

pub struct Validator {
    cfg: Config,
    srcs: HashMap<String, Zam>,
}

impl Validator {
    pub fn new(cfg: Config, srcs: HashMap<String, Zam>) -> Self {
        Self { cfg, srcs }
    }

    pub fn validate(mut self, mut err: usize) {
        self.main_fn();
        self.identifier();

        for v in self.srcs.values() {
            err += v.parser.err
        }

        if err != 0 {
            err!(
                "could not compile `{}` due to {err} previous error{}",
                self.cfg.pkg.name,
                match err {
                    1 => "",
                    _ => "s",
                }
            )
        }
    }

    pub fn lookup_dec(&self, id: &String) -> Vec<&String> {
        let mut tmp = Vec::new();

        for (path, src) in &self.srcs {
            if src.block.dec.contains_key(id) {
                tmp.push(path);
            }
        }

        tmp
    }
}
