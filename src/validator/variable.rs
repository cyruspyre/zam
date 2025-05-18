use crate::zam::Entity;

use super::{lookup::Lookup, Validator};

impl Validator {
    pub fn variable<'a>(&mut self, val: &mut Entity, lookup: &mut Lookup<'a>) {
        let Entity::Variable { exp, done, .. } = val else {
            return;
        };

        if *done {
            return;
        }

        *done = true;
        lookup.typ(&mut exp.typ.kind);
        self.validate_type(exp, lookup);

        println!("{exp} is {}", exp.typ);
    }
}
