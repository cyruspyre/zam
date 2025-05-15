use crate::{
    misc::Bypass,
    zam::{block::Block, statement::Statement, Entity},
};

use super::{lookup::Lookup, Validator};

impl Validator {
    pub fn block(&mut self, block: &mut Block, lookup: &mut Lookup) {
        let dec = &mut block.dec;
        let Lookup { var, stack, cur } = lookup.bypass();

        stack.push(dec.bypass());

        for (id, val) in dec.bypass() {
            cur.rng = id.rng;

            match val {
                //Entity::Type { typ, public } => todo!(),
                Entity::Variable { .. } => self.variable(val, lookup),
                Entity::Struct { .. } => self.r#struct(val, lookup),
                Entity::Function { .. } => self.fun(val, lookup),
            }
        }

        // debugging purpose
        let mut tmp = dec.into_iter();
        while let Some((_, Entity::Variable { exp, .. })) = tmp.next() {
            println!("{exp} is {}", exp.typ)
        }

        let len = var.len();

        for v in &mut block.stm {
            match v {
                Statement::Variable { id, data } => {
                    // println!("{exp}");
                    self.variable(data, lookup);
                    var.insert(id.bypass(), data.bypass());
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

        // let mut tmp = block.stm.bypass().into_iter();

        // while let Some(Statement::Variable { exp, .. }) = tmp.next() {
        //     println!("{exp} is {}", exp.typ)
        // }

        stack.pop();
        var.truncate(len);
    }
}
