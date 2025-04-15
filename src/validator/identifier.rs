use indexmap::IndexMap;

use crate::{
    misc::Bypass,
    zam::{block::Hoistable, statement::Statement},
};

use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn identifier(&mut self) {
        // variables defined in local scope
        let mut var = IndexMap::new();
        let mut stack = Vec::with_capacity(self.srcs.len());
        let lookup = &mut Lookup {
            cur: None,
            var: var.bypass(),
            stack: stack.bypass(),
        };

        for src in self.bypass().srcs.values_mut() {
            let cur = &mut src.parser;
            let block = src.block.bypass();
            let dec = &mut block.dec;
            lookup.cur = Some(cur.bypass());

            stack.push(src.block.dec.bypass());

            for val in dec.bypass().values_mut() {
                match val {
                    Hoistable::Variable { exp, .. } => {
                        self.variable(cur, Entity::Variable(exp), lookup)
                    }
                    Hoistable::Struct { fields, .. } => {
                        self.r#struct(Entity::Struct(fields), lookup)
                    }
                    _ => todo!(),
                }
            }

            // debugging purpose
            let mut tmp = dec.into_iter();
            while let Some((_, Hoistable::Variable { exp, .. })) = tmp.next() {
                println!("{exp} is {}", exp.typ)
            }

            for v in &mut block.stm {
                match v {
                    Statement::Variable { name, exp, .. } => {
                        var.insert(name.bypass(), exp.bypass());
                    }
                    _ => todo!(),
                }
            }
        }
    }
}
