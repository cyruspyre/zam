use std::borrow::Cow;

use crate::{
    misc::Bypass,
    parser::{
        log::{Log, Point},
        span::ToSpan,
    },
    zam::{
        expression::{misc::Range, term::Term, Expression},
        typ::{kind::TypeKind, Type},
    },
};

use super::{lookup::Lookup, Validator};

impl Validator {
    pub fn validate_type<'a>(&mut self, exp: &mut Expression, lookup: &mut Lookup) -> Option<()> {
        let kind = exp.typ.kind.bypass();
        let cur = lookup.cur.bypass();
        let mut typ: Option<Cow<TypeKind>> = None;
        let mut iter = exp.bypass().data.iter_mut().enumerate();

        while let Some((i, v)) = iter.next() {
            cur.rng = v.rng;

            let kind = match v.data.bypass() {
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
                // todo: find a way to apply inferred type to used variables in an expr
                Term::Identifier(id) => {
                    lookup
                        .bypass()
                        .as_typ(id.span(v.rng), || match iter.next() {
                            Some(v) => Some(&mut v.1.data),
                            _ => None,
                        })?
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

            cur.log(&mut pnt, Log::Error, "type mismatch", "");
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
