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
    zam::{Zam, block::Impls, typ::Type},
};

pub struct Analyzer {
    pub cfg: Config,
    pub root: Zam,
    pub impls: Impls,
    pub cur: NaiveMap<ThreadId, ThreadStorage>,
}

pub struct ThreadStorage {
    zam: RefMut<Zam>,
    /// Stack of recently accessed block types for use in `return`/`break` type inference
    rets: Vec<Ref<Type>>,
}

impl Analyzer {
    pub fn validate(mut self) {
        self.main_fn();

        let tmp = self.bypass();
        let name = &tmp.cfg.pkg.name;
        let mut err = Vec::<&_>::new();
        let mut stack = vec![&mut tmp.root];

        while let Some(zam) = stack.pop() {
            zam.lookup.stamp = Ref(&zam.id);

            self.cur.insert(
                current().id(),
                ThreadStorage {
                    zam: RefMut(zam),
                    rets: Vec::new(),
                },
            );
            self.block(&mut zam.block, None);
            self.cur.pop();
            err.push(zam.log.err.bypass());

            for v in zam.bypass().mods.values_mut() {
                v.parent = RefMut(zam);
                stack.push(v);
            }
        }

        let err = err.into_iter().map(|v| *v).sum::<usize>();

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

    pub fn cur_full(&mut self) -> &mut ThreadStorage {
        self.cur.get(&current().id()).unwrap()
    }

    pub fn cur(&mut self) -> &mut Zam {
        &mut self.cur_full().zam
    }

    pub fn set_tmp_cur(&mut self, new: &mut Zam) -> CustomDrop<impl FnMut() + use<>> {
        let cur = &mut self.cur.get(&current().id()).unwrap().bypass().zam;
        let tmp = cur.0;

        cur.0 = new;

        CustomDrop(move || cur.0 = tmp)
    }
}
