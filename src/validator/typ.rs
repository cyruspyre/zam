use crate::{
    misc::Bypass,
    parser::{
        log::{Log, Point},
        Parser,
    },
    zam::{
        expression::{term::Term, Expression},
        typ::kind::TypeKind,
    },
};

use super::Validator;

impl Validator {
    pub fn validate_type(&mut self, cur: &mut Parser, exp: &mut Expression) -> Option<()> {
        let kind = exp.typ.kind.bypass();
        let mut typ: Option<TypeKind> = None;
        let mut iter = exp.data.iter_mut().peekable();
        let mut num = Vec::new();

        while let Some(v) = iter.next() {
            cur.rng = v.rng;

            let tmp = v.data.bypass();
            let mut kind = match tmp.bypass() {
                Term::Integer { bit, sign, .. } => {
                    if *bit == 0 {
                        num.push(tmp);
                    }

                    TypeKind::Integer {
                        bit: *bit,
                        sign: *sign,
                    }
                }
                Term::Float { bit, .. } => {
                    if *bit == 0 {
                        num.push(tmp);
                    }

                    TypeKind::Float(*bit)
                }
                Term::Add | Term::Sub | Term::Mod => match &typ {
                    Some(v) => v.clone(),
                    _ => cur.err("expected a term beforehand")?,
                },
                _ => todo!(),
            };

            let tmp = match &mut typ {
                Some(TypeKind::Unknown) | None => {
                    typ = Some(kind);
                    continue;
                }
                Some(v) => v,
            };

            let mut sim_typ = [tmp.bypass(), &mut kind];

            for (i, v) in sim_typ.bypass().iter_mut().enumerate() {
                let tmp = &mut sim_typ[sim_typ.len() - i - 1];

                match v {
                    TypeKind::Float(0) if matches!(tmp, TypeKind::Float(v) if *v != 0) => {}
                    TypeKind::Integer {
                        bit: 0,
                        sign: sign_,
                    } if matches!(tmp, TypeKind::Integer { bit, sign } if {
                        *bit != 0 && sign == sign_ || *sign && !*sign_
                    }) => {}
                    _ => continue,
                }

                **v = tmp.clone()
            }

            if *tmp != kind {
                cur.log(
                    &[(
                        cur.rng,
                        Point::Error,
                        format!("expected `{tmp}`, found `{kind}`"),
                    )],
                    Log::Error,
                    "type mismatch",
                    "",
                );
                return None;
            }
        }

        let typ = match typ? {
            TypeKind::Float(0) => TypeKind::Float(32),
            TypeKind::Integer { bit: 0, .. } => TypeKind::Integer {
                bit: 32,
                sign: true,
            },
            v => v,
        };

        if kind.data == TypeKind::Unknown {
            kind.data = typ.clone()
        } else if typ != kind.data {
            cur.log(
                &[
                    (kind.rng, Point::Info, "inferred from here"),
                    (
                        exp.exp_rng(),
                        Point::Error,
                        &format!("expected `{kind}`, found `{typ}`"),
                    ),
                ],
                Log::Error,
                "type mismatch",
                "",
            );
        }

        let (a, b) = match typ {
            TypeKind::Integer { bit, sign } => (bit, sign),
            _ => (32, true),
        };
        let c = match typ {
            TypeKind::Float(v) => v,
            _ => 32,
        };

        for v in num {
            match v {
                Term::Integer { bit, sign, .. } => {
                    *bit = a;
                    *sign = b;
                }
                Term::Float { bit, .. } => *bit = c,
                _ => unreachable!(),
            }
        }
        println!("{exp}");

        Some(())
    }
}
