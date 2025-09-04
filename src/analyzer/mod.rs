mod block;
mod expression;
mod fun;
pub mod lookup;
mod main_fn;
mod misc;
mod r#struct;
mod typ;
mod variable;

use std::thread::{ThreadId, current};

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, CustomDrop, Ref, RefMut},
    naive_map::NaiveMap,
    zam::{Zam, block::Impls},
};

pub struct Project {
    pub cfg: Config,
    pub root: Zam,
    pub impls: Impls,
    pub cur: NaiveMap<ThreadId, RefMut<Zam>>,
}

impl Project {
    pub fn validate(mut self) {
        self.main_fn();

        let tmp = self.bypass();
        let name = &tmp.cfg.pkg.name;
        let mut err = 0;
        let mut stack = vec![&mut tmp.root];

        while let Some(zam) = stack.pop() {
            zam.lookup.stamp = Ref(&zam.id);
            self.cur.insert(current().id(), RefMut(zam));
            self.block(&mut zam.block, None);
            self.cur.pop();

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

    pub fn cur(&mut self) -> &mut Zam {
        self.cur.get(&current().id()).unwrap()
    }

    pub fn set_tmp_cur(&mut self, new: &mut Zam) -> CustomDrop<impl FnMut() + use<>> {
        let cur = self.cur.get(&current().id()).unwrap().bypass();
        let tmp = cur.0;

        cur.0 = new;

        CustomDrop(move || cur.0 = tmp)
    }
}
