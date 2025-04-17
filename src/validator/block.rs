use crate::{
    misc::Bypass,
    zam::{
        block::{Block, Hoistable},
        statement::Statement,
    },
};

use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn block(&mut self, block: &mut Block, lookup: &mut Lookup) {
        let dec = &mut block.dec;
        let Lookup { var, stack, .. } = lookup.bypass();

        stack.push(dec.bypass());

        for val in dec.bypass().values_mut() {
            match val {
                Hoistable::Variable { exp, .. } => self.variable(Entity::Variable(exp), lookup),
                Hoistable::Struct { fields, .. } => self.r#struct(Entity::Struct(fields), lookup),
                Hoistable::Function {
                    arg, ret, block, ..
                } => self.fun(Entity::Function { arg, ret, block }, lookup),
            }
        }

        // debugging purpose
        let mut tmp = dec.into_iter();
        while let Some((_, Hoistable::Variable { exp, .. })) = tmp.next() {
            println!("{exp} is {}", exp.typ)
        }

        let len = var.len();

        for v in &mut block.stm {
            match v {
                Statement::Variable { name, exp, .. } => {
                    var.insert(name.bypass(), exp.bypass());
                }
                _ => todo!(),
            }
        }

        stack.pop();
        var.truncate(len);
    }
}
