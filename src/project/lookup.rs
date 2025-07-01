use std::{borrow::Cow, collections::VecDeque, ops::DerefMut};

use strsim::jaro;

use crate::{
    log::{Log, Point},
    misc::{Bypass, Ref, RefMut},
    parser::span::Span,
    project::Project,
    zam::{
        Entity,
        expression::{misc::Range, term::Term},
        identifier::Identifier,
        typ::kind::TypeKind,
    },
};

impl Project {
    pub fn lookup(
        &mut self,
        id: &Identifier,
        required: bool,
    ) -> Option<(&Identifier, &mut Entity)> {
        let mut zam = self.cur().deref_mut().bypass();
        let lookup = zam.lookup.bypass();
        let qualified = id.is_qualified();

        if qualified {
            let mut tmp = zam.bypass();
            let mut idx = 0;

            while idx != id.len() {
                let leaf = &id[idx];

                idx += 1;
                tmp = match leaf.as_str() {
                    "self" => tmp,
                    "super" if tmp.parent.is_null() => zam
                        .log
                        .err_rng(id.rng(), format!("`{}` doesn't have parent", tmp.id))?,
                    "super" => tmp.parent.deref_mut(),
                    v if let Some(v) = zam.mods.get_mut(v) => v.bypass(),
                    _ if idx != id.len() => zam
                        .log
                        .err_rng(leaf.rng, format!("`{}` doesn't have `{leaf}`", zam.id))?,
                    _ => break,
                };
            }

            zam = tmp
        }

        lookup.stamp = (Ref(&zam.id), lookup.decs.len());

        let leaf = id.leaf_name();

        if let Some((_, k, v)) = lookup.vars.bypass().get_full_mut(leaf) {
            return Some((k, v.deref_mut()));
        }

        let mut two = VecDeque::new();
        let mut iter = lookup.decs.bypass().iter_mut();

        loop {
            let (idx, dec) = match iter.next() {
                Some((idx, map)) => (**idx, map.deref_mut()),
                _ if lookup.decs.is_empty() => (0, zam.block.dec.bypass()),
                _ => break,
            };

            if let Some((_, k, v)) = dec.bypass().get_full_mut(leaf) {
                return Some((k, v));
            }

            if required {
                two.push_back(dec[idx..].iter_mut().map(|(k, v)| (k, v)))
            }
        }

        if !required {
            return None;
        }

        let mut one = lookup.vars.iter_mut().map(|(k, v)| (&*k, v.deref_mut()));
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
                let sim = jaro(leaf, k.leaf_name());

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

        for v in zam.mods.values_mut() {
            if let Some((i, id_cur, _)) = v.block.dec.get_full(leaf) {
                let suggestion = id_cur.qualify(&v.id);
                let [msg, note] = if v.block.public.contains(&i) {
                    [
                        format!("did you mean `{suggestion}`?"),
                        format!("qualify it as `{suggestion}` or import it"),
                    ]
                } else {
                    [
                        String::new(),
                        format!("`{suggestion}` exists but is private"),
                    ]
                };

                zam.log.call(
                    &mut [(id.rng(), Point::Error, "")],
                    Log::Error,
                    format!("cannot find `{leaf}`. {msg}"),
                    note,
                )
            }
        }

        if (0.8..=1.0).contains(&res.0) {
            todo!()
        }

        None
    }

    pub fn as_typ<'a, F>(&mut self, id_exp: &Identifier, mut next: F) -> Option<Cow<TypeKind>>
    where
        F: FnMut() -> Option<&'a mut Term>,
    {
        let log = self.cur().log.bypass();
        let res = self.bypass().lookup(id_exp, true);
        let (id_entity, val) = res?;

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
                generic,
                fields,
                impls,
                traits,
            } => {
                let Some(Term::Struct(vals)) = next() else {
                    log.err(format!(
                        "expected struct initalization, found type `{id_entity}`"
                    ))?
                };
                let mut pnt = Vec::new();
                let err = log.err;

                self.r#struct(id_entity, val);

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
                        "unknown field{} for struct `{id_entity}`",
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
                    log.err_rng(id_exp.rng(), msg);
                }

                if err != log.err {
                    return None;
                }

                Cow::Owned(TypeKind::Entity {
                    id: Ref(id_entity),
                    data: RefMut(val),
                })
            }
            v => todo!("Entity::{v:?}"),
        };

        Some(typ)
    }

    pub fn qualify_type(&mut self, kind: &mut Span<TypeKind>) {
        let id = match kind.data.bypass() {
            TypeKind::ID(id) => id,
            TypeKind::Tuple(fields) => {
                return for v in fields {
                    self.qualify_type(&mut v.kind)
                };
            }
            _ => return,
        };

        let mut label = None;
        let msg = if let Some((id, val)) = self.lookup(&id, false) {
            let entity = match val {
                Entity::Struct { .. } => {
                    return kind.data = TypeKind::Entity {
                        id: Ref(id),
                        data: RefMut(val),
                    };
                }
                _ => val.name(),
            };

            format!("expected type found `{entity}`")
        } else {
            label = kind.try_as_number();

            if label.is_none() && !matches!(kind.data, TypeKind::ID(_)) {
                return;
            }

            format!("cannot find type `{id}`")
        };

        if label.is_none() && !id.is_qualified() {
            let id = &id[0].data;
            let [a, b] = ["isize", "usize"].map(|v| jaro(id, v));
            let res = if a >= 0.8 {
                "usize"
            } else if b >= 0.8 {
                "usize"
            } else {
                ""
            };

            if res != "" {
                label = Some(format!("did you mean `{res}`?"))
            }
        }

        let pnt = &mut [(id.rng(), Point::Error, label.unwrap_or_default())];

        self.cur().log.call(pnt, Log::Error, msg, "");
    }
}
