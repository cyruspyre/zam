use std::borrow::Cow;

use crate::{
    log::{Log, Point},
    misc::Bypass,
    project::Project,
    zam::{
        expression::{misc::Range, term::Term, Expression},
        typ::{kind::TypeKind, Type},
    },
};

impl Project {
    pub fn validate_type<'a>(&mut self, exp: &mut Expression) -> Option<()> {
        let kind = exp.typ.kind.bypass();

        debug_assert!(
            !matches!(kind.data, TypeKind::ID(_)),
            "expression `{exp}` expects a raw type id `{kind}` at {}",
            self.location(exp.data.rng())
        );

        let log = self.cur().zam.log.bypass();
        let mut typ: Option<Cow<TypeKind>> = None;
        let mut iter = exp.bypass().data.iter_mut().enumerate();

        while let Some((i, v)) = iter.next() {
            log.rng = v.rng;

            let kind = match v.data.bypass() {
                Term::As(Type {
                    kind,
                    sub,
                    ptr,
                    null,
                    ..
                }) => {
                    self.typ(kind);

                    if !matches!(
                        kind.data,
                        TypeKind::Bool | TypeKind::Integer { .. } | TypeKind::Float(_)
                    ) {
                        log.err_rng(
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
                    _ => log.err("expected a term beforehand")?,
                },
                // todo: find a way to apply inferred type to used variables in an expr
                Term::Identifier(id) => self.bypass().as_typ(id, || match iter.next() {
                    Some(v) => Some(&mut v.1.data),
                    _ => None,
                })?,
                Term::Tuple(vals) => {
                    let mut buf = Vec::with_capacity(vals.len());
                    let a = match &kind.data {
                        TypeKind::Tuple(v) => Some(v),
                        _ => None,
                    };

                    for (i, exp) in vals.iter_mut().enumerate() {
                        if let Some(v) = a {
                            if let Some(typ) = v.get(i) {
                                exp.typ = typ.clone()
                            }
                        }

                        self.validate_type(exp);
                        buf.push(exp.typ.clone());
                    }

                    Cow::Owned(TypeKind::Tuple(buf))
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

            if coercible(typ, &kind) {
                *typ = kind;
                continue;
            } else if coercible(&kind, typ) {
                continue;
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

            log.bypass()(
                &mut [(
                    log.rng,
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

        if coercible(&typ, &Cow::Borrowed(&kind.data)) {
            return Some(());
        }

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

            log(&mut pnt, Log::Error, "type mismatch", "");
        }

        Some(())
    }
}

fn coercible<'a>(a: &Cow<TypeKind>, b: &Cow<TypeKind>) -> bool {
    match a.as_ref() {
        TypeKind::Float(0) if matches!(b.as_ref(), TypeKind::Float(v) if *v != 0) => {}
        TypeKind::Integer {
            bit: 0,
            sign: sign_,
        } if matches!(b.as_ref(), TypeKind::Integer { bit, sign } if {
            *bit != 0 && sign == sign_ || *sign && !*sign_
        }) => {}
        _ => return false,
    }

    true
}
