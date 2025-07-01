use std::ops::DerefMut;

use crate::{
    log::{Log, Point},
    misc::Bypass,
    project::Project,
    zam::{Entity, Zam, expression::misc::Range, identifier::Identifier, typ::kind::TypeKind},
};

impl Project {
    pub fn r#struct<'a>(&mut self, id: &Identifier, val: &mut Entity) {
        let Entity::Struct {
            fields,
            done,
            impls,
            ..
        } = val
        else {
            return;
        };

        if *done {
            return;
        }

        let zam = self.cur().deref_mut().bypass();
        let Zam { log, lookup, .. } = zam;
        let stamp = lookup.stamp;
        let __ = log.bypass().ctx(id.rng(), Point::Info, "in this struct");

        for typ in fields.values_mut() {
            let kind = &mut typ.kind;

            self.qualify_type(kind);

            if matches!(kind.data, TypeKind::Entity { id: id_, .. } if id_.rng() == id.rng())
                && stamp == lookup.stamp
                && typ.null == 0
                && !typ.raw
            {
                return log(
                    &mut [(kind.rng, Point::Error, "")],
                    Log::Error,
                    "recursive type detected without indirections",
                    "make it nullable or wrap it in a type that provides indirections",
                );
            }
        }

        *done = true;

        // todo: this will always return true lol so fix it
        if !zam.block.global {
            return;
        }

        let Some(map) = self.bypass().impls.get(&id.leaf_name().data) else {
            return;
        };

        for (zam_id, val) in map {
            let mut idx = 0;
            let mut iter = zam_id.iter();
            let first = iter.next().unwrap();
            let mut cur = if self.cfg.pkg.name == **first {
                self.root.bypass()
            } else {
                todo!("`ZamPath` for dependencies")
            };

            for id in iter {
                cur = cur.mods.get_mut(&**id).unwrap();
            }

            let tmp = zam.bypass();
            self.cur().0 = cur;

            let log = zam.log.bypass();

            while let Some(([one, two], generic, block)) = val.get(idx) {
                idx += 1;
                let Some(res) = self.lookup(one, true) else {
                    continue;
                };

                if !two.is_empty() {
                    todo!("implement traits for types")
                }
            }

            self.cur().0 = tmp
        }
    }
}
