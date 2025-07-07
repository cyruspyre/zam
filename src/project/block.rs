use std::ops::DerefMut;

use crate::{
    misc::{Bypass, Ref, RefMut},
    project::Project,
    zam::{Entity, Lookup, block::Block, expression::misc::Range, statement::Statement, typ::Type},
};

impl Project {
    pub fn block(&mut self, block: &mut Block, ret: Option<&Type>) {
        let zam = self.cur().deref_mut().bypass();
        let dec = block.dec.bypass();
        let Lookup { vars, decs, .. } = zam.lookup.bypass();
        let len = vars.len();
        let mut idx = 0;

        decs.push((Ref(&idx), RefMut(dec.bypass())));

        for (id, val) in dec.bypass() {
            zam.log.rng = id.rng();

            match val {
                Entity::Trait { .. } => todo!(),
                //Entity::Type { typ, public } => todo!(),
                Entity::Variable { .. } => self.variable(val),
                Entity::Struct { .. } => self.r#struct(id, val),
                Entity::Function { .. } => self.fun(val),
            }

            vars.insert(Ref(id), RefMut(val));
            idx += 1
        }

        decs.pop();

        for v in &mut block.stm.iter_mut() {
            match v {
                Statement::Variable { id, data } => {
                    self.variable(data);
                    vars.insert(Ref(id.bypass()), RefMut(data.bypass()));
                }
                Statement::Expression(exp) => {
                    self.assert_expr(exp);
                }
                Statement::Return(exp) => {
                    if let Some(typ) = ret {
                        exp.typ = typ.clone()
                    }

                    self.assert_expr(exp);
                }
                v => todo!("Statement::{v:?}"),
            }
        }

        vars.truncate(len);
    }
}
