mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod r#struct;
mod typ;
mod variable;

use std::path::Path;

use indexmap::IndexMap;
use lookup::{Current, Lookup};

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    zam::{block::Block, identifier::Identifier, typ::generic::Generic, Zam},
};

pub struct Project {
    pub cfg: Config,
    pub root: Zam,
    #[rustfmt::skip]
    pub impls: IndexMap<Identifier, IndexMap<Ref<Path>, IndexMap<Identifier, Vec<(Generic, Block)>>>>,
}

impl Project {
    pub fn validate(mut self) {
        self.main_fn();

        let tmp = self.bypass();
        let name = &tmp.cfg.pkg.name;
        let mut err = 0;
        let mut stack = vec![(name, &mut tmp.root)];

        while let Some((id, zam)) = stack.pop() {
            let mut lookup = Lookup {
                project: self.bypass(),
                cur: Current {
                    id,
                    zam: zam.bypass(),
                },
                var: IndexMap::new(),
                stack: Vec::new(),
            };

            self.block(&mut zam.block, &mut lookup);

            err += zam.parser.err;

            for v in &mut zam.mods {
                stack.push(v);
            }
        }

        if err != 0 {
            err!(
                "could not compile `{}` due to {err} previous error{}",
                name,
                match err {
                    1 => "",
                    _ => "s",
                }
            )
        }
    }
}
