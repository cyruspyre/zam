use crate::{
    misc::Bypass,
    parser::{span::ToSpan, Context},
    zam::Entity,
};

use super::{lookup::Lookup, Project};

impl Project {
    pub fn r#struct<'a>(&mut self, val: &mut Entity, lookup: &mut Lookup<'a>) {
        let Entity::Struct { fields, .. } = val else {
            return;
        };
        let cur = lookup.cur.zam.parser.bypass();

        cur.ctx = Some(Context::Struct.span(cur.rng));

        for v in fields.values_mut() {
            lookup.typ(&mut v.kind);
        }

        cur.ctx = None;
    }
}
