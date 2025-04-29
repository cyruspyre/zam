use std::borrow::Cow;

use crate::{
    misc::Bypass,
    parser::log::{Log, Point},
    zam::{
        expression::{misc::Range, term::Term, Expression},
        typ::kind::TypeKind,
    },
};

use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn validate_type<'a>(&mut self, exp: &mut Expression, lookup: &mut Lookup) -> Option<()> {
        exp.done = true;

        let kind = exp.typ.kind.bypass();
        let cur = lookup.cur.bypass();
        let mut typ: Option<Cow<TypeKind>> = None;
        let mut iter = exp.data.iter_mut();
        let mut num = Vec::new();

        while let Some(v) = iter.next() {
            cur.rng = v.rng;

            let mut kind = match v.data.bypass() {
                Term::Bool(_) => Cow::Owned(TypeKind::Bool),
                Term::Integer { bit, sign, .. } => Cow::Owned(TypeKind::Integer {
                    bit: *bit,
                    sign: *sign,
                }),
                Term::Float { bit, .. } => Cow::Owned(TypeKind::Float(*bit)),
                Term::Add | Term::Sub | Term::Div | Term::Mod => match typ.bypass() {
                    Some(v) => Cow::Borrowed(unsafe { &*(v.as_ref() as *const _) }),
                    _ => cur.err("expected a term beforehand")?,
                },
                Term::Identifier(id) => 'a: {
                    let res = lookup.bypass().call(id);
                    let mut pnt = Vec::new();

                    if let Some(Ok((_, mut v))) = res {
                        break 'a Cow::Borrowed(match v.bypass() {
                            Entity::Variable(exp) => {
                                if exp.done && exp.typ.kind.data == TypeKind::Unknown {
                                    kind.data = TypeKind::Unknown;
                                    return None;
                                }

                                self.variable(v, lookup);
                                num.push(exp.typ.kind.data.bypass());
                                &exp.typ.kind.data
                            }
                            _ => todo!(),
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

            if *typ != kind {
                cur.log(
                    &mut [(
                        cur.rng,
                        Point::Error,
                        format!("expected `{typ}`, found `{kind}`"),
                    )],
                    Log::Error,
                    "type mismatch",
                    "",
                );
                return None;
            }
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
