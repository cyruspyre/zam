use std::{collections::VecDeque, ops::DerefMut};

use strsim::jaro;

use crate::{
    analyzer::Project,
    log::{Log, Point},
    misc::{Bypass, Ref, RefMut},
    parser::span::{Span, ToSpan},
    zam::{
        Entity,
        expression::{misc::Range, term::Term},
        identifier::Identifier,
        typ::{Type, kind::TypeKind},
    },
};

impl Project {
    pub fn lookup(
        &mut self,
        id: &Identifier,
        required: bool,
    ) -> Option<(&Identifier, &mut Entity)> {
        let mut zam = self.cur().bypass();
        let zam_id = zam.id.bypass();
        let log = zam.log.bypass();
        let qualified = id.is_qualified();

        if qualified {
            let mut cur = zam.bypass();
            let mut idx = 0;

            while idx != id.len() {
                let leaf = &id[idx];
                let tmp = leaf.as_str();

                idx += 1;
                cur = match tmp {
                    "self" => cur,
                    "super" if cur.parent.is_null() => {
                        log.err_rng(id.rng(), format!("`{}` doesn't have parent module", cur.id))?
                    }
                    "super" => cur.parent.deref_mut(),
                    v if let Some(v) = zam.mods.get_mut(v) => v.bypass(),
                    _ if tmp == self.cfg.pkg.name => self.root.bypass(),
                    _ if idx != id.len() => {
                        log.err_rng(leaf.rng, format!("`{}` doesn't have `{leaf}`", zam.id))?
                    }
                    _ => break,
                };
            }

            zam = cur
        }

        let lookup = zam.lookup.bypass();
        let leaf = id.leaf_name();
        // Meaning of `res.0` (similarity score):
        //   - INFINITY: private declaration
        //   - MAX: declared in another file
        //   - ≥ 0.8: consider valid similarity match
        //   - < 0.8: ignore similarity
        let mut res: (f64, Option<(&Identifier, &Entity)>) = (0.0, None);

        // real programmer way
        macro_rules! cond_return {
            ($id:expr, $val:expr) => {
                match zam.block.id_is_public($id) {
                    true => return Some(($id, $val)),
                    _ => res = (f64::INFINITY, Some(($id, $val))),
                }
            };
        }

        lookup.stamp = Ref(&zam.id);

        if let Some((_, id, v)) = lookup.vars.bypass().get_full_mut(leaf) {
            cond_return!(id, v)
        }

        let mut two = VecDeque::new();
        let mut iter = lookup.decs.bypass().iter_mut();

        if res.0.is_finite() {
            loop {
                let (idx, dec) = match iter.next() {
                    Some((idx, map)) => (**idx, map.deref_mut()),
                    _ if lookup.decs.is_empty() && two.is_empty() => (0, zam.block.dec.bypass()),
                    _ => break,
                };

                if let Some((_, k, v)) = dec.bypass().get_full_mut(leaf) {
                    cond_return!(k, v);
                    break;
                }

                if required {
                    two.push_back(dec[idx..].iter_mut().map(|(k, v)| (k, v)))
                }
            }
        }

        if !required {
            return None;
        }

        let mut one = lookup.vars.iter_mut().map(|(k, v)| (&*k, v.deref_mut()));

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

        for cur in zam.mods.values_mut() {
            if let Some(v) = cur.block.dec.bypass().get_key_value(leaf) {
                let sim = match cur.block.id_is_public(v.0) {
                    true => f64::MAX,
                    _ => f64::INFINITY,
                };

                res = (sim, Some(v));
                zam = cur;

                break;
            }
        }

        let msg = format!("cannot find `{id}`");
        let mut pnt = Vec::new();

        if res.0 < 0.8 {
            res.1 = None
        }

        let [label, note] = if let Some((id, val)) = res.1 {
            let entity = val.name();
            let (id, extra) = if res.0 != f64::MAX && *zam_id == zam.id {
                let tmp = format!("{entity} defined here");

                pnt.push((id.rng(), Point::Info, tmp));
                (id, String::new())
            } else {
                (&id.qualify(&zam.id), format!(" in `{}`", zam.id))
            };

            if let f64::INFINITY = res.0 {
                [String::new(), format!("`{id}` exists but is private")]
            } else {
                let tmp = match res.0 {
                    f64::MAX => format!("qualify it as `{id}` or import it"),
                    _ => String::new(),
                };

                [format!("did you mean {entity} `{leaf}`{extra}?"), tmp]
            }
        } else {
            [const { String::new() }; 2]
        };

        pnt.push((id.rng(), Point::Error, label));
        log(&mut pnt, Log::Error, msg, note);

        None
    }

    pub fn qualify_identifier<'a, F>(&mut self, id_exp: &Identifier, mut next: F) -> Option<Type>
    where
        F: FnMut() -> Option<&'a mut Term>,
    {
        let log = self.cur().log.bypass();
        let res = self.bypass().lookup(id_exp, true);
        let (id_entity, val) = res?;

        log.rng = id_exp.rng();

        let typ = match val.bypass() {
            Entity::Variable { exp, done, .. } => {
                if *done && exp.typ.kind.data == TypeKind::Unknown {
                    return None;
                }

                self.variable(val);
                exp.typ.clone()
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
                        self.assert_expr(exp)?
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
                    log.err(msg);
                }

                if err != log.err {
                    return None;
                }

                Type {
                    kind: TypeKind::Entity {
                        id: Ref(id_entity),
                        data: RefMut(val),
                    }
                    .span([0; 2]),
                    ..Default::default()
                }
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
