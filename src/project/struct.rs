use crate::{
    misc::Bypass,
    parser::{span::ToSpan, Context},
    zam::{Entity, Zam},
};

impl Zam {
    pub fn r#struct<'a>(&mut self, val: &mut Entity) {
        let Entity::Struct { fields, done, .. } = val else {
            return;
        };

        if *done {
            return;
        }

        let cur = self.parser.bypass();

        cur.ctx = Some(Context::Struct.span(cur.rng));

        for v in fields.values_mut() {
            self.typ(&mut v.kind);
        }

        cur.ctx = None;
        *done = true;
    }
}
