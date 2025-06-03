use crate::{
    misc::Bypass,
    parser::{span::ToSpan, Context},
    project::Project,
    zam::Entity,
};

impl Project {
    pub fn r#struct<'a>(&mut self, val: &mut Entity) {
        let Entity::Struct { fields, done, .. } = val else {
            return;
        };

        if *done {
            return;
        }

        let log = self.cur().log.bypass();

        log.ctx = Some(Context::Struct.span(log.rng));

        for v in fields.values_mut() {
            self.typ(&mut v.kind);
        }

        log.ctx = None;
        *done = true;
    }
}
