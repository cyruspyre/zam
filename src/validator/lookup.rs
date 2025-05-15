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
    zam::{typ::kind::TypeKind, Entity},
};

pub struct Lookup<'a> {
    pub cur: &'a mut Parser,
    pub var: IndexMap<&'a Span<String>, &'a mut Entity>,
    pub stack: Vec<&'a mut IndexMap<Span<String>, Entity>>,
}

impl<'a> Lookup<'a> {
    pub fn call(&mut self, id: &String) -> Option<Result<(&Span<String>, &mut Entity)>> {
        if let Some((_, k, v)) = self.var.bypass().get_full_mut(id) {
            return Some(Ok((*k, v.deref_mut().into())));
        }

        let mut one = self.var.iter_mut().map(|(k, v)| (*k, v.deref_mut()));
        let mut two = VecDeque::new();

        for dec in self.stack.deref_mut() {
            if let Some((_, k, v)) = dec.bypass().get_full_mut(id) {
                return Some(Ok((k, v.into())));
            }

            two.push_back(dec.iter_mut().map(|(k, v)| (k, v)));
        }

        let mut res: (f64, Option<(&Span<String>, &mut Entity)>) = (0.0, None);
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
        let cur = self.cur.bypass();

        cur.rng = kind.rng;

        let TypeKind::ID(id) = kind.data.bypass() else {
            return;
        };
        let res = self.call(id);
        let mut label = None;

        if matches!(res, None | Some(Err(_)))
            && {
                label = kind.try_as_number();
                label.is_none()
            }
            && !matches!(kind.data, TypeKind::ID(_))
        {
            return;
        }

        let mut pnt = Vec::new();
        let Some(res) = res else {
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
        let name = match v {
            Entity::Variable { .. } => "variable",
            Entity::Function { .. } => "function",
            Entity::Struct { .. } if ok => {
                return todo!(); // kind.data = TypeKind::Dec(v);
            }
            _ => todo!(),
        };

        pnt.push((k.rng, Point::Info, format!("{name} defined here")));

        let recursive = *id == k.data;
        let msg = if recursive {
            "recursive type detected without indirections".into()
        } else if ok {
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

        return cur.log(&mut pnt, Log::Error, msg, note);
    }
}
