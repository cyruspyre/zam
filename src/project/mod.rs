mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod misc;
mod r#struct;
mod typ;
mod variable;

use std::thread::{current, ThreadId};

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, RefMut},
    zam::{block::Impls, Zam},
};

pub struct Project {
    pub cfg: Config,
    pub root: Zam,
    pub impls: Impls,
    pub cur: Vec<(ThreadId, RefMut<Zam>)>,
}

impl Project {
    pub fn validate(mut self) {
        self.main_fn();

        let tmp = self.bypass();
        let name = &tmp.cfg.pkg.name;
        let mut err = 0;
        let mut stack = vec![&mut tmp.root];

        while let Some(zam) = stack.pop() {
            self.cur.push((current().id(), RefMut(zam)));
            self.block(&mut zam.block);
            self.cur.swap_remove(self.cur_idx());

            err += zam.log.err;

            for v in zam.bypass().mods.values_mut() {
                v.parent = RefMut(zam);
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

    fn cur_idx(&self) -> usize {
        let id = current().id();

        self.cur.iter().position(|v| v.0 == id).unwrap()
    }

    pub fn cur(&mut self) -> &mut Zam {
        &mut self.bypass().cur[self.cur_idx()].1
    }
}
