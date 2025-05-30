use crate::zam::{Entity, Zam};

impl Zam {
    pub fn variable<'a>(&mut self, val: &mut Entity) {
        let Entity::Variable { exp, done, .. } = val else {
            return;
        };

        if *done {
            return;
        }

        *done = true;
        self.typ(&mut exp.typ.kind);
        self.validate_type(exp);

        println!("{exp} is {}", exp.typ);
    }
}
