use crate::{
    log::{Log, Point},
    misc::Bypass,
    project::{Current, Project},
    zam::{expression::misc::Range, identifier::Identifier, Entity},
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

        let Current { zam, global } = self.cur().bypass();
        let log = &mut zam.log;

        // log.ctx = Some(Context::Struct.span(log.rng));

        for v in fields.values_mut() {
            self.typ(&mut v.kind);
        }

        log.ctx = None;
        *done = true;

        if !*global {
            return;
        }

        let Some(map) = self.bypass().impls.get_mut(&id.leaf_name().data) else {
            return;
        };

        dbg!(&map);

        for (zam_id, val) in map {
            let mut idx = 0;
            let mut iter = zam_id.iter();
            let first = iter.next().unwrap();
            let mut zam = if self.cfg.pkg.name == **first {
                self.root.bypass()
            } else {
                todo!("`ZamPath` for dependencies")
            };

            for id in iter {
                zam = &mut zam.mods[&**id];
            }

            let log = &mut zam.log;

            while let Some(([one, two], gen, block)) = val.get(idx) {
                if !zam.block.dec.contains_key(one) {
                    log(
                        &mut [(one.rng(), Point::Error, "")],
                        Log::Error,
                        format!("cannot find `{one}`. did you mean `{id}`?"),
                        format!("qualify it as `{}` or import it", id.relative(zam_id)),
                    );
                }
                println!("{one} {:?}", self.lookup(one));
                if !two.is_empty() {
                    todo!("implement traits for types")
                }
                idx += 1;
            }
        }
    }
}
