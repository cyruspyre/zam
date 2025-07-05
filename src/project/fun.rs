use crate::{project::Project, zam::Entity};

impl Project {
    pub fn fun(&mut self, val: &mut Entity) {
        let Entity::Function {
            arg, ret, block, ..
        } = val
        else {
            return;
        };

        for v in arg.values_mut() {
            self.qualify_type(&mut v.kind);
        }

        self.qualify_type(&mut ret.kind);

        let Some(block) = block else { return };

        self.block(block, Some(ret));
    }
}
