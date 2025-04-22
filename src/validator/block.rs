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
        let Lookup { var, stack, cur } = lookup.bypass();

        stack.push(dec.bypass());

        for (id, val) in dec.bypass() {
            cur.rng = id.rng;

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
                    self.variable(exp.bypass().into(), lookup);
                    var.insert(name.bypass(), exp.bypass());
                }
                Statement::Conditional { cond, default } => {
                    for (exp, block) in cond {
                        self.validate_type(exp, lookup);
                        self.block(block, lookup);
                    }

                    if let Some(block) = default {
                        self.block(block, lookup);
                    }
                }
                v => todo!("Statement::{v:?}"),
            }
        }

        let mut tmp = block.stm.bypass().into_iter();

        while let Some(Statement::Variable { exp, .. }) = tmp.next() {
            println!("{exp} is {}", exp.typ)
        }

        stack.pop();
        var.truncate(len);
    }
}
