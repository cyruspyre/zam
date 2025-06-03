use std::{borrow::Cow, collections::VecDeque, ops::DerefMut};

use strsim::jaro;

use crate::{
    log::{Log, Point},
    misc::{Bypass, Either, Ref, RefMut, Result},
    parser::span::Span,
    project::Project,
    zam::{
        expression::{misc::Range, term::Term},
        identifier::Identifier,
        typ::kind::TypeKind,
        Entity,
    },
};

impl Project {
    pub fn lookup(&mut self, id: &Identifier) -> Option<Result<(&Identifier, &mut Entity)>> {
        let mut zam = self.cur();

        for key in id.iter() {
            let Some(next) = zam.mods.get_mut(&key.data) else {
                break;
            };

            zam = next
        }

        let lookup = zam.lookup.bypass();
        let id = id.leaf_name();

        if let Some((_, k, v)) = lookup.vars.bypass().get_full_mut(id) {
            return Some(Ok((k, v.deref_mut().into())));
        }

        let mut one = lookup.vars.iter_mut().map(|(k, v)| (&*k, v.deref_mut()));
        let mut two = VecDeque::new();
        let mut iter = lookup.decs.iter_mut();

        loop {
            let dec = match iter.next() {
                Some(v) => &mut **v,
                _ if two.is_empty() => zam.block.dec.bypass(),
                _ => break,
            };

            if let Some((_, k, v)) = dec.bypass().get_full_mut(id) {
                return Some(Ok((k, v.into())));
            }

            two.push_back(dec.iter_mut().map(|(k, v)| (k, v)));
        }

        let mut res: (f64, Option<(&Identifier, &mut Entity)>) = (0.0, None);
        let mut tmp: Option<(&Identifier, _)> = None;

        loop {
            if let Some((k, v)) = one.next() {
                tmp = Some((&k, v))
            } else if let Some(v) = two.front_mut() {
                if let Some(v) = v.next() {
                    tmp = Some(v);
                } else {
                    two.pop_front();
                }
            }

            if let Some((k, v)) = tmp.take() {
                let sim = jaro(id, k.leaf_name());

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

    pub fn as_typ<'a, F>(&mut self, id: &Identifier, mut next: F) -> Option<Cow<TypeKind>>
    where
        F: FnMut() -> Option<&'a mut Term>,
    {
        let log = self.cur().log.bypass();
        let res = self.bypass().lookup(id);
        let Some(Ok((k, val))) = res else {
            let mut pnt = Vec::new();

            if let Some(Err((k, v))) = res {
                pnt.push((
                    k.rng(),
                    Point::Info,
                    format!("similar {} named `{k}` exists", v.name()),
                ));
            }

            pnt.push((id.rng(), Point::Error, String::new()));
            log(
                &mut pnt,
                Log::Error,
                format!("cannot find identifier `{id}`"),
                "",
            );
            return None;
        };

        let typ = match val.bypass() {
            Entity::Variable { exp, done, .. } => 'a: {
                if *done && exp.typ.kind.data == TypeKind::Unknown {
                    break 'a Cow::Owned(TypeKind::Unknown);
                }

                self.variable(val);
                Cow::Borrowed(&exp.typ.kind.data)
            }
            Entity::Struct {
                done,
                gen,
                fields,
                impls,
                traits,
            } => {
                let Some(Term::Struct(vals)) = next() else {
                    log.err(format!("expected struct initalization, found type `{id}`"))?
                };
                let mut pnt = Vec::new();
                let err = log.err;

                self.r#struct(val);

                for (k, exp) in &mut *vals {
                    if let Some(v) = fields.get(k) {
                        exp.typ = v.clone();
                        exp.typ.kind.rng.fill(0);
                        self.validate_type(exp)?
                    } else {
                        pnt.push((k.rng(), Point::Error, ""));
                    }
                }

                if pnt.len() != 0 {
                    let msg = format!(
                        "unknown field{} for struct `{id}`",
                        if pnt.len() == 1 { "" } else { "s" }
                    );

                    log(&mut pnt, Log::Error, msg, "");
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
                    log.err_rng(id.rng(), msg);
                }

                if err != log.err {
                    return None;
                }

                Cow::Owned(TypeKind::Entity {
                    id: Ref(k),
                    data: RefMut(val),
                })
            }
            v => todo!("Entity::{v:?}"),
        };

        Some(typ)
    }

    pub fn typ(&mut self, kind: &mut Span<TypeKind>) {
        let log = self.cur().log.bypass();

        log.rng = kind.rng;

        let kind = &mut kind.data;
        let TypeKind::ID(id) = kind.bypass() else {
            if let TypeKind::Tuple(v) = kind {
                for typ in v {
                    self.typ(&mut typ.kind)
                }
            }

            return;
        };

        let res = self.lookup(id);
        let mut label = None;

        if matches!(res, None | Some(Err(_)))
            && {
                label = kind.try_as_number();
                label.is_none()
            }
            && !matches!(kind, TypeKind::ID(_))
        {
            return;
        }

        let mut pnt = Vec::new();
        let Some(res) = res else {
            // ehh try to refactor it
            log.bypass()(
                &mut [(log.rng, Point::Error, label.unwrap_or_default())],
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
                return *kind = TypeKind::Entity {
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

        pnt.push((log.rng, Point::Error, label));

        return log(&mut pnt, Log::Error, msg, note);
    }
}
