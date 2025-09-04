use crate::{analyzer::Project, zam::Entity};

impl Project {
    pub fn variable<'a>(&mut self, val: &mut Entity) {
        let Entity::Variable { exp, done, .. } = val else {
            return;
        };

        if *done {
            return;
        }

        *done = true;
        self.qualify_type(&mut exp.typ.kind);
        self.assert_expr(exp);

        println!("{exp} is {}", exp.typ);
    }
}
