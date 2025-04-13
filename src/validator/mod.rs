mod identifier;
mod main_fn;
mod r#struct;
mod typ;
mod variable;
mod lookup;

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
}
