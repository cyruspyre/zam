use std::{borrow::Cow, collections::VecDeque, ops::DerefMut};

use indexmap::IndexMap;
use strsim::jaro;

use crate::{
    misc::{Bypass, Either, Ref, RefMut, Result},
    parser::{
        log::{Log, Point},
        span::Span,
        Parser,
    },
    zam::{
        expression::{misc::Range, term::Term},
        identifier::Identifier,
        typ::kind::TypeKind,
        Entity,
    },
};

use super::{Project, ZamID};

pub struct Lookup<'a> {
    pub(super) cur: Current<'a>,
    pub validator: &'a mut Project,
    pub var: IndexMap<&'a Identifier, &'a mut Entity>,
    pub stack: Vec<&'a mut IndexMap<Identifier, Entity>>,
}

pub(super) struct Current<'a> {
    pub id: &'a ZamID,
    pub parser: &'a mut Parser,
}

impl<'a> Lookup<'a> {
    pub fn call(&mut self, id: &Identifier) -> Option<Result<(&Identifier, &mut Entity)>> {
        dbg!(&self.validator.srcs.keys());
        if id.is_qualified() {
            todo!("qualified identifier lookup")
        }

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

        let mut res: (f64, Option<(&Identifier, &mut Entity)>) = (0.0, None);
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
                let sim = jaro(id.leaf_name(), k.leaf_name());

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

    pub fn as_typ<F>(&mut self, id: &Identifier, mut next: F) -> Option<Cow<TypeKind>>
    where
        F: FnMut() -> Option<&'a mut Term>,
    {
        let cur = self.cur.parser.bypass();
        let validator = self.validator.bypass();
        let res = self.bypass().call(id);
        let Some(Ok((k, v))) = res else {
            let mut pnt = Vec::new();

            if let Some(Err((k, v))) = res {
                pnt.push((
                    k.rng(),
                    Point::Info,
                    format!("similar {} named `{k}` exists", v.name()),
                ));
            }

            pnt.push((id.rng(), Point::Error, String::new()));
            cur.log(
                &mut pnt,
                Log::Error,
                format!("cannot find identifier `{id}`"),
                "",
            );
            return None;
        };

        let typ = match v.bypass() {
            Entity::Variable { exp, done, .. } => 'a: {
                if *done && exp.typ.kind.data == TypeKind::Unknown {
                    break 'a Cow::Owned(TypeKind::Unknown);
                }

                validator.variable(v, self);
                Cow::Borrowed(&exp.typ.kind.data)
            }
            Entity::Struct {
                gen,
                fields,
                impls,
                traits,
            } => {
                let Some(Term::Struct(vals)) = next() else {
                    cur.err(format!("expected struct initalization, found type `{id}`"))?
                };
                let mut pnt = Vec::new();
                let err = cur.err;

                for (k, exp) in &mut *vals {
                    if let Some(v) = fields.get(k) {
                        exp.typ = v.clone();
                        exp.typ.kind.rng.fill(0);
                        validator.validate_type(exp, self)?
                    } else {
                        pnt.push((k.rng(), Point::Error, ""));
                    }
                }

                if pnt.len() != 0 {
                    let msg = format!(
                        "unknown field{} for struct `{id}`",
                        if pnt.len() == 1 { "" } else { "s" }
                    );

                    cur.log(&mut pnt, Log::Error, msg, "");
                }

                let mut msg = String::from("missing field");
                let last = match fields.len() {
                    0 | 1 => 1,
                    n => {
                        msg.push('s');
                        n - 1
                    }
                };

                for (i, id) in fields.keys().enumerate() {
                    if !vals.contains_key(id) {
                        msg += if i == last {
                            " and "
                        } else if i != 0 {
                            ", "
                        } else {
                            " "
                        };

                        msg += &format!("`{id}`");
                    }
                }

                if msg.len() > 14 {
                    cur.err_rng(id.rng(), msg);
                }

                if err != cur.err {
                    return None;
                }

                Cow::Owned(TypeKind::Entity {
                    id: Ref(k),
                    data: RefMut(v),
                })
            }
            v => todo!("Entity::{v:?}"),
        };

        Some(typ)
    }

    pub fn typ(&mut self, kind: &mut Span<TypeKind>) {
        let cur = self.cur.parser.bypass();

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
                return kind.data = TypeKind::Entity {
                    id: Ref(k),
                    data: RefMut(v),
                };
            }
            _ => todo!(),
        };

        pnt.push((k.rng(), Point::Info, format!("{name} defined here")));

        let recursive = id == k;
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
            let b = [k.leaf_name(), "isize", "usize"]
                .map(|v| (jaro(v, id.leaf_name()), v))
                .into_iter()
                .max_by(|a, b| jaro(a.1, id.leaf_name()).total_cmp(&jaro(b.1, id.leaf_name())))
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
