use std::borrow::Cow;

use crate::{
    misc::Bypass,
    parser::log::{Log, Point},
    zam::{
        expression::{misc::Range, term::Term, Expression},
        typ::{kind::TypeKind, Type},
        Entity,
    },
};

use super::{lookup::Lookup, Validator};

impl Validator {
    pub fn validate_type<'a>(&mut self, exp: &mut Expression, lookup: &mut Lookup) -> Option<()> {
        exp.done = true;

        let kind = exp.typ.kind.bypass();
        let cur = lookup.cur.bypass();
        let mut typ: Option<Cow<TypeKind>> = None;
        let mut iter = exp.bypass().data.iter_mut().enumerate();
        let mut num = Vec::new();

        while let Some((i, v)) = iter.next() {
            cur.rng = v.rng; //struct u8; 1u8 as u8;

            let mut kind = match v.data.bypass() {
                Term::As(Type {
                    kind,
                    sub,
                    ptr,
                    null,
                    ..
                }) => {
                    lookup.typ(kind);

                    if !matches!(
                        kind.data,
                        TypeKind::Bool | TypeKind::Integer { .. } | TypeKind::Float(_)
                    ) {
                        cur.err_rng(
                            [exp.data[i - 1].rng[0], v.rng[1]],
                            "non-primitive type casting",
                        )?
                    }
                    todo!()
                }
                Term::Bool(_) => Cow::Owned(TypeKind::Bool),
                Term::Integer { bit, sign, .. } => Cow::Owned(TypeKind::Integer {
                    bit: *bit,
                    sign: *sign,
                }),
                Term::Float { bit, .. } => Cow::Owned(TypeKind::Float(*bit)),
                Term::Add | Term::Sub | Term::Mul | Term::Div | Term::Mod => match typ.bypass() {
                    Some(v) => Cow::Borrowed(unsafe { &*(v.as_ref() as *const _) }),
                    _ => cur.err("expected a term beforehand")?,
                },
                Term::Identifier(id) => 'a: {
                    let res = lookup.bypass().call(id);
                    let mut pnt = Vec::new();

                    if let Some(Ok((k, v))) = res {
                        break 'a Cow::Borrowed(match v.bypass() {
                            Entity::Variable { exp, .. } => {
                                if exp.done && exp.typ.kind.data == TypeKind::Unknown {
                                    kind.data = TypeKind::Unknown;
                                    return None;
                                }

                                self.variable(v, lookup);
                                num.push(exp.typ.kind.data.bypass());
                                &exp.typ.kind.data
                            }
                            Entity::Struct { fields, .. } => {
                                let mut lol = || {
                                    match &mut iter.next()?.1.data {
                                        Term::Struct(vals) => {
                                            let err = cur.err;
                                            let mut pnt = Vec::new();
                                            let mut idx = Vec::new();

                                            for (k, v) in vals {
                                                if let Some((i, _, typ)) = fields.get_full(k) {
                                                    v.typ = typ.clone();
                                                    idx.push(i);
                                                } else {
                                                    pnt.push((k.rng, Point::Error, ""));
                                                }
                                            }

                                            let mut msg = String::from("missing field");
                                            let last = match fields.len() {
                                                1 => 1,
                                                n => {
                                                    msg.push('s');
                                                    n - 1
                                                }
                                            };
                                            let tmp = msg.len();

                                            for (i, v) in fields.keys().enumerate() {
                                                if match idx.get(i) {
                                                    Some(n) => i != *n,
                                                    _ => true,
                                                } {
                                                    msg += if i == last {
                                                        " and "
                                                    } else if i != 0 {
                                                        ", "
                                                    } else {
                                                        " "
                                                    };

                                                    msg += &format!("`{v}`");
                                                }
                                            }

                                            if msg.len() != tmp {
                                                cur.err(msg);
                                            }

                                            if !pnt.is_empty() {
                                                let msg = format!(
                                                    "unknown field{} for struct `{k}`",
                                                    if pnt.len() == 1 { "" } else { "s" }
                                                );
                                                cur.log(&mut pnt, Log::Error, msg, "");
                                            }

                                            if err != cur.err {
                                                return None;
                                            }
                                        }
                                        Term::Tuple(vals) => {}
                                        _ => return None,
                                    }

                                    Some(())
                                };

                                if lol().is_none() {
                                    cur.err(format!(
                                        "expected struct initalization, found type `{id}`"
                                    ))?
                                }
                                // match &iter.next().unwrap().1.data {
                                //     Term::Struct(vals) => {}
                                //     Term::Tuple(vals) => {}
                                //     _ => cur.err("expected struct intialization, found type")?,
                                // };
                                todo!()
                            }
                            v => todo!("Entity::{v:?}"),
                        });
                    } else if let Some(Err((k, v))) = res {
                        let msg = format!("similar {} named `{k}` exists", v.name());

                        pnt.push((k.rng, Point::Info, msg));
                    }

                    pnt.push((cur.rng, Point::Error, String::new()));

                    cur.log(
                        &mut pnt,
                        Log::Error,
                        format!("cannot find identifier `{id}`"),
                        "",
                    );
                    return None;
                }
                v => todo!("Term::{v:?}"),
            };
            let typ = match &mut typ {
                Some(v) => v,
                _ => {
                    typ = Some(kind);
                    continue;
                }
            };

            let mut sim_typ = [typ.bypass(), &mut kind];

            for (i, v) in sim_typ.bypass().iter_mut().enumerate() {
                let tmp = &mut sim_typ[sim_typ.len() - i - 1];

                match &***v {
                    TypeKind::Float(0) if matches!(&***tmp, TypeKind::Float(v) if *v != 0) => {}
                    TypeKind::Integer {
                        bit: 0,
                        sign: sign_,
                    } if matches!(&***tmp, TypeKind::Integer { bit, sign } if {
                        *bit != 0 && sign == sign_ || *sign && !*sign_
                    }) => {}
                    _ => continue,
                }

                **v = tmp.clone()
            }

            if *typ == kind {
                continue;
            }

            let one = typ.to_string();
            let two = kind.to_string();
            let note = if one == two {
                &format!("they have similar names but are acutally distinct types")
            } else {
                ""
            };

            cur.log(
                &mut [(
                    cur.rng,
                    Point::Error,
                    format!("expected `{one}`, found `{two}`"),
                )],
                Log::Error,
                "type mismatch",
                note,
            );
            return None;
        }

        let typ = typ?;

        if kind.data == TypeKind::Unknown {
            kind.data = typ.into_owned()
        } else if *typ != kind.data {
            let mut pnt = Vec::with_capacity(2);

            if kind.rng[1] != 0 {
                pnt.push((kind.rng, Point::Info, "inferred from here".into()));
            }

            pnt.push((
                exp.data.rng(),
                Point::Error,
                format!("expected `{kind}`, found `{typ}`"),
            ));

            cur.log(&mut pnt, Log::Error, "type mismatch", "");
        }

        for v in num {
            *v = kind.data.clone()
        }

        Some(())
    }
}
