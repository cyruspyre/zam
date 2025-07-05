use crate::{
    misc::Bypass,
    project::Project,
    zam::{
        expression::{Expression, misc::Range, term::Term},
        typ::{Type, kind::TypeKind},
    },
};

impl Project {
    pub fn assert_expr<'a>(&mut self, exp: &mut Expression) -> Option<()> {
        let expected = exp.typ.bypass();

        debug_assert!(
            !matches!(expected.kind.data, TypeKind::ID(_)),
            "expression `{exp}` expects a raw type id `{}` at {}",
            expected.kind,
            self.location(exp.data.rng())
        );

        let log = self.cur().log.bypass();
        let mut inferred: Option<Type> = None;
        let mut fuck = Type::default();
        let mut iter = exp.bypass().data.iter_mut();

        while let Some(term) = iter.next() {
            if fuck.kind.rng[0] == 0 {
                fuck.kind.rng = term.rng
            } else {
                fuck.kind.rng[1] = term.rng[1]
            }

            fuck.kind.data = match term.data.bypass() {
                Term::As(typ) => {
                    self.qualify_type(&mut typ.kind);

                    if !matches!(
                        typ.kind.data,
                        TypeKind::Bool | TypeKind::Integer { .. } | TypeKind::Float(_)
                    ) {
                        log.err_rng(fuck.kind.rng, "non-primitive type casting")?
                    }

                    todo!()
                }
                Term::Bool(_) => TypeKind::Bool,
                Term::Integer { bit, sign, .. } => TypeKind::Integer {
                    bit: *bit,
                    sign: *sign,
                },
                Term::Float { bit, .. } => TypeKind::Float(*bit),
                Term::Add | Term::Sub | Term::Mul | Term::Div | Term::Mod => match inferred {
                    Some(_) => {
                        fuck = Type::default();
                        continue;
                    }
                    _ => log.err_rng(term.rng, "expected a term beforehand")?,
                },
                // todo: find a way to apply inferred type to used variables in an expr
                Term::Identifier(id) => {
                    let tmp = self.bypass().qualify_identifier(id, || match iter.next() {
                        Some(v) => Some(&mut v.data),
                        _ => None,
                    })?;

                    // tf man
                    fuck.null += tmp.null;
                    fuck.ptr += tmp.ptr;
                    fuck.raw = tmp.raw;
                    fuck.sub = tmp.sub;

                    tmp.kind.data
                }
                Term::Group(exp) => {
                    self.assert_expr(exp);

                    let tmp = &exp.typ;

                    // man honestly wtf
                    fuck.null += tmp.null;
                    fuck.ptr += tmp.ptr;
                    fuck.raw = tmp.raw;
                    fuck.sub = tmp.sub.clone();

                    tmp.kind.data.clone()
                }
                Term::Tuple(vals) => {
                    let mut buf = Vec::with_capacity(vals.len());

                    for exp in vals {
                        self.assert_expr(exp);
                        buf.push(exp.typ.clone());
                    }

                    TypeKind::Tuple(buf)
                }
                Term::Ref => {
                    fuck.ptr += 1;
                    continue;
                }
                Term::Deref => {
                    fuck.ptr -= 1;

                    continue;
                }
                v => todo!("Term::{v:?}"),
            };

            if fuck.ptr.is_negative() {
                fuck.ptr = (fuck.ptr + 1).abs();
                log.err_rng(fuck.kind.rng, format!("cannot dereference `{fuck}`"));
                return None;
            }

            let inferred = match &mut inferred {
                Some(v) => v,
                _ => {
                    inferred = Some(fuck.clone());
                    continue;
                }
            };

            self.assert_type(&mut fuck, inferred);
        }
        let inferred = &mut inferred?;
        let tmp = self.assert_type(inferred, expected);

        tmp
    }
}
