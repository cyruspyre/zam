use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn fun(&mut self, val: Entity, lookup: &mut Lookup) {
        let Entity::Function { arg, ret, block } = val else {
            return;
        };

        for v in arg.values_mut() {
            lookup.typ(&mut v.kind);
        }

        lookup.typ(&mut ret.kind);
    }
}
