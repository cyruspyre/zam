mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod r#struct;
mod typ;
mod variable;

use indexmap::IndexMap;
use lookup::Lookup;

use crate::{cfg::Config, err, misc::Bypass, zam::Zam};

pub struct Validator {
    cfg: Config,
    srcs: IndexMap<String, Zam>,
}

impl Validator {
    pub fn new(cfg: Config, srcs: IndexMap<String, Zam>) -> Self {
        Self { cfg, srcs }
    }

    pub fn validate(mut self, mut err: usize) {
        self.main_fn();

        for src in self.bypass().srcs.values_mut() {
            let mut lookup = Lookup {
                cur: &mut src.parser,
                var: IndexMap::new(),
                stack: Vec::new(),
            };

            self.block(&mut src.block, &mut lookup);
            err += lookup.cur.err;
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
