use std::{collections::VecDeque, ops::DerefMut};

use indexmap::IndexMap;
use strsim::jaro;

use crate::{
    misc::{Bypass, Either, Result},
    parser::{
        log::{Log, Point},
        span::Span,
        Parser,
    },
    zam::{
        block::{Block, Hoistable},
        expression::Expression,
        fields::Fields,
        typ::{kind::TypeKind, Type},
    },
};

#[derive(Debug)]
pub enum Entity<'a> {
    Variable(&'a mut Expression),
    Struct(&'a mut Fields<Type>),
    Function {
        arg: &'a mut Fields<Type>,
        ret: &'a mut Type,
        block: &'a mut Option<Block>,
    },
}

impl<'a> Entity<'a> {
    pub fn name(&self) -> &str {
        match self {
            Entity::Struct(_) => "struct",
            Entity::Variable(_) => "variable",
            Entity::Function { .. } => "function",
        }
    }
}

impl<'a> From<&'a mut Hoistable> for Entity<'a> {
    fn from(value: &'a mut Hoistable) -> Self {
        match value {
            Hoistable::Variable { exp, .. } => Entity::Variable(exp),
            Hoistable::Struct { fields, .. } => Entity::Struct(fields),
            Hoistable::Function {
                arg, ret, block, ..
            } => Entity::Function { arg, ret, block },
        }
    }
}

impl<'a> From<&'a mut Expression> for Entity<'a> {
    fn from(value: &'a mut Expression) -> Self {
        Entity::Variable(value)
    }
}

pub struct Lookup<'a> {
    pub cur: &'a mut Parser,
    pub var: IndexMap<&'a Span<String>, &'a mut Expression>,
    pub stack: Vec<&'a mut IndexMap<Span<String>, Hoistable>>,
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

    pub fn typ(&mut self, kind: &mut Span<TypeKind>) {
        let label = kind.bypass().try_as_number();
        let cur = self.cur.bypass();

        cur.rng = kind.rng;

        let TypeKind::ID(id) = &kind.data else {
            return;
        };
        let mut pnt = Vec::new();
        let Some(res) = self.call(id) else {
            // ehh try to refactor it
            cur.log(
                &mut [(cur.rng, Point::Error, label.unwrap_or_default())],
                Log::Error,
                format!("cannot find type `{id}`"),
                "",
            );
            return;
        };
        let ok = res.is_ok();
        let (k, v) = res.either();
        let name = v.name();

        pnt.push((k.rng, Point::Info, format!("{name} defined here")));

        let recursive = *id == k.data;
        let msg = if recursive {
            "recursive type detected without indirections".into()
        } else if ok {
            if name.is_empty() {
                return;
            }

            format!("expected type, found {name} `{k}`")
        } else {
            format!("cannot find type `{id}`")
        };
        let label = if let Some(v) = label {
            v
        } else if recursive {
            "creates an infinite-sized type".into()
        } else {
            let b = [k.as_str(), "isize", "usize"]
                .map(|v| (jaro(v, id), v))
                .into_iter()
                .max_by(|a, b| jaro(a.1, id).total_cmp(&jaro(b.1, id)))
                .unwrap();

            if b.0 >= 0.8 && !name.is_empty() {
                format!("did you mean `{}`?", b.1)
            } else {
                "not a type".into()
            }
        };
        let note = match recursive {
            true => "make it nullable or wrap it in a type that provides indirections",
            _ => "",
        };

        pnt.push((kind.rng, Point::Error, label));

        cur.log(&mut pnt, Log::Error, msg, note);
        return;
    }
}
