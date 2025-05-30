mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod misc;
mod r#struct;
mod typ;
mod variable;

use std::path::Path;

use indexmap::IndexMap;

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

        let name = &self.cfg.pkg.name;
        let mut err = 0;
        let mut stack = vec![&mut self.root];

        while let Some(zam) = stack.pop() {
            zam.bypass().block(&mut zam.block);

            err += zam.parser.err;

            for v in zam.mods.values_mut() {
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
