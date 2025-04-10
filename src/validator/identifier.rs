use indexmap::IndexMap;
use strsim::jaro;

use crate::{
    misc::{Bypass, Result},
    parser::span::Span,
    zam::{block::Hoistable, statement::Statement},
};

use super::Validator;

pub struct Lookup<'a> {
    stm: &'a mut IndexMap<Span<String>, Hoistable>,
    stack: &'a mut Vec<&'a mut IndexMap<Span<String>, Hoistable>>,
}

impl<'a> Lookup<'a> {
    pub fn call(&mut self, id: &String) -> Option<Result<(&Span<String>, &mut Hoistable)>> {
        let mut lol = Some(&mut self.stm);
        let mut iter = self.stack.iter_mut();
        let mut tmp = (0.0, None);

        while let Some(v) = lol {
            if let Some((_, k, v)) = v.bypass().get_full_mut(id) {
                return Some(Ok((k, v)));
            }

            for (k, v) in v.iter_mut() {
                let score = jaro(id, k);

                if score > tmp.0 {
                    tmp = (score, Some((k, v)));
                }
            }

            lol = iter.next();
        }

        match tmp.0 {
            0.8..=1.0 => Some(Err(tmp.1?)),
            _ => None,
        }
    }
}

impl Validator {
    pub fn identifier(&mut self) {
        let mut stm: IndexMap<Span<String>, _> = IndexMap::new();
        let mut stack: Vec<&mut IndexMap<Span<_>, _>> = Vec::with_capacity(self.srcs.len());
        let mut lookup = Lookup {
            stm: stm.bypass(),
            stack: stack.bypass(),
        };

        for src in self.bypass().srcs.values_mut() {
            let cur = &mut src.parser;
            let block = src.block.bypass();
            let dec = &mut block.dec;

            stack.push(src.block.dec.bypass());

            for (id, v) in &mut *dec {
                let tmp = match v {
                    Hoistable::Variable { .. } => self.variable(cur, v, &mut lookup),
                    _ => {}
                };
            }

            // debugging purpose
            let mut tmp = dec.into_iter();
            while let Some((_, Hoistable::Variable { val, .. })) = tmp.next() {
                println!("{val} is {}", val.typ)
            }

            for v in &mut block.stm {
                match v {
                    Statement::Variable { name, val, cte } => {
                        // tried using reference without cloning but its too much pain
                        // the cost of cloning should be negligble
                        stm.insert(name.clone(), Hoistable::VarRef(&mut val.typ));
                    }
                    _ => todo!(),
                }
            }
        }
    }
}
