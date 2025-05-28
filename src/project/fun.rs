use crate::zam::Entity;

use super::{lookup::Lookup, Project};

impl Project {
    pub fn fun(&mut self, val: &mut Entity, lookup: &mut Lookup) {
        let Entity::Function {
            arg, ret, block, ..
        } = val
        else {
            return;
        };

        for v in arg.values_mut() {
            lookup.typ(&mut v.kind);
        }

        lookup.typ(&mut ret.kind);

        if let Some(v) = &mut *block {
            self.block(v, lookup);
        }
    }
}
