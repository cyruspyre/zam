use crate::{
    misc::{Bypass, Ref, RefMut},
    project::Project,
    zam::{block::Block, expression::misc::Range, statement::Statement, Entity, Lookup},
};

impl Project {
    pub fn block(&mut self, block: &mut Block) {
        let cur = self.cur().bypass();
        let dec = &mut block.dec;
        let Lookup { vars, decs } = cur.lookup.bypass();

        decs.push(RefMut(dec.bypass()));

        for (id, val) in dec.bypass() {
            cur.log.rng = id.rng();

            match val {
                //Entity::Type { typ, public } => todo!(),
                Entity::Variable { .. } => self.variable(val),
                Entity::Struct { .. } => self.r#struct(val),
                Entity::Function { .. } => self.fun(val),
            }
        }

        let len = vars.len();

        for v in &mut block.stm {
            match v {
                Statement::Variable { id, data } => {
                    self.variable(data);
                    vars.insert(Ref(id.bypass()), RefMut(data.bypass()));
                }
                Statement::Conditional { cond, default } => {
                    for (exp, block) in cond {
                        self.validate_type(exp);
                        self.block(block);
                    }

                    if let Some(block) = default {
                        self.block(block);
                    }
                }
                v => todo!("Statement::{v:?}"),
            }
        }

        decs.pop();
        vars.truncate(len);
    }
}
