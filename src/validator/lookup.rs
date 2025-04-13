use std::{collections::VecDeque, ops::DerefMut};

use indexmap::IndexMap;
use strsim::jaro;

use crate::{
    misc::{Bypass, Result},
    parser::span::Span,
    zam::{block::Hoistable, expression::Expression, fields::Fields, typ::Type},
};

#[derive(Debug)]
pub enum Entity<'a> {
    Variable(&'a mut Expression),
    Struct(&'a mut Fields<Type>),
}

impl<'a> Entity<'a> {
    pub fn name(&self) -> &str {
        match self {
            Entity::Struct(_) => "struct",
            Entity::Variable(_) => "variable",
        }
    }
}

impl<'a> From<&'a mut Hoistable> for Entity<'a> {
    fn from(value: &'a mut Hoistable) -> Self {
        match value {
            Hoistable::Variable { exp, .. } => Entity::Variable(exp),
            Hoistable::Struct { fields, .. } => Entity::Struct(fields),
            _ => unreachable!(),
        }
    }
}

impl<'a> From<&'a mut Expression> for Entity<'a> {
    fn from(value: &'a mut Expression) -> Self {
        Entity::Variable(value)
    }
}

pub struct Lookup<'a> {
    pub var: &'a mut IndexMap<&'a Span<String>, &'a mut Expression>,
    pub stack: &'a mut Vec<&'a mut IndexMap<Span<String>, Hoistable>>,
}

impl<'a> Lookup<'a> {
    pub fn call(&mut self, id: &String) -> Option<Result<(&Span<String>, Entity)>> {
        if let Some((_, k, v)) = self.var.bypass().get_full_mut(id) {
            return Some(Ok((*k, v.deref_mut().into())));
        }

        let mut one = self.var.iter_mut().map(|(k, v)| (*k, v.deref_mut().into()));
        let mut two = VecDeque::new();

        for dec in self.stack.deref_mut() {
            if let Some((_, k, v)) = dec.bypass().get_full_mut(id) {
                return Some(Ok((k, v.into())));
            }

            two.push_back(dec.iter_mut().map(|(k, v)| (k, Entity::from(v))));
        }

        let mut res: (f64, Option<(&Span<String>, Entity)>) = (0.0, None);
        let mut tmp = None;

        loop {
            if let Some(v) = one.next() {
                tmp = Some(v)
            } else if let Some(v) = two.front_mut() {
                if let Some(v) = v.next() {
                    tmp = Some(v);
                } else {
                    two.pop_front();
                }
            }

            if let Some((k, v)) = tmp.take() {
                let sim = jaro(id, k);

                if sim > res.0 {
                    res = (sim, Some((k, v)))
                }

                if sim == 1.0 {
                    break;
                }
            } else {
                break;
            }
        }

        match res.0 {
            0.8..=1.0 => Some(Err(res.1?)),
            _ => None,
        }
    }
}
