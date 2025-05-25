mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod r#struct;
mod typ;
mod variable;

use indexmap::IndexMap;
use lookup::Lookup;

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    zam::{block::Impls, Zam},
};

pub struct Validator {
    pub cfg: Config,
    pub cur: Ref<String>,
    pub srcs: IndexMap<String, Zam>,
    pub impls: IndexMap<Ref<String>, Impls>,
}

impl Validator {
    pub fn validate(mut self, mut err: usize) {
        self.main_fn();

        for (id, src) in &mut self.bypass().srcs {
            let mut lookup = Lookup {
                validator: self.bypass(),
                cur: &mut src.parser,
                var: IndexMap::new(),
                stack: Vec::new(),
            };

            self.cur = Ref(id);
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
