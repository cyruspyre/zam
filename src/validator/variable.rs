use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn variable<'a>(&mut self, val: Entity<'a>, lookup: &mut Lookup<'a>) {
        let Entity::Variable(exp) = val else {
            return;
        };

        if exp.done {
            return;
        }

        lookup.typ(&mut exp.typ.kind);
        self.validate_type(exp, lookup);
    }
}
