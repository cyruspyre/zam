use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn r#struct<'a>(&mut self, val: Entity<'a>, lookup: &mut Lookup<'a>) {
        let Entity::Struct(fields) = val else {
            return;
        };

        for v in fields.values_mut() {
            lookup.typ(&mut v.kind);
        }
    }
}
