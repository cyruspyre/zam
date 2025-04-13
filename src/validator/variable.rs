use strsim::jaro;

use crate::{
    misc::{Bypass, Either},
    parser::{
        log::{Log, Point},
        Parser,
    },
    zam::typ::kind::TypeKind,
};

use super::{
    lookup::{Entity, Lookup},
    Validator,
};

impl Validator {
    pub fn variable<'a>(&mut self, cur: &mut Parser, val: Entity<'a>, lookup: &mut Lookup<'a>) {
        let Entity::Variable(exp) = val else {
            return;
        };

        if exp.done {
            return;
        }

        let kind = exp.typ.kind.bypass();
        let rng = kind.rng;
        let label = kind.bypass().try_as_number();

        'two: {
            let TypeKind::ID(id) = &kind.data else {
                break 'two;
            };
            let mut pnt = Vec::new();
            let Some(res) = lookup.call(id) else {
                cur.err(format!("cannot find type `{id}`"));
                break 'two;
            };
            let ok = res.is_ok();
            let (k, v) = res.either();
            let name = 'lol: {
                let name = match v {
                    // Entity::Function { .. } => "function",
                    Entity::Struct { .. } => "struct",
                    _ => break 'lol "",
                };
                let tmp = format!("{name} defined here");

                pnt.push((k.rng, Point::Info, tmp));
                name
            };
            let msg = if ok {
                if name.is_empty() {
                    break 'two;
                }

                format!("expected type, found {name} `{k}`",)
            } else {
                format!("cannot find type `{id}`")
            };
            let label = if let Some(v) = label {
                v
            } else {
                let b = [k.as_str(), "isize", "usize"]
                    .map(|v| (jaro(v, id), v))
                    .into_iter()
                    .max_by(|a, b| jaro(a.1, id).total_cmp(&jaro(b.1, id)))
                    .unwrap();

                if b.0 >= 0.8 && !name.is_empty() {
                    format!("did you mean `{}`?", b.1)
                } else {
                    "not a type".into()
                }
            };

            pnt.push((rng, Point::Error, label));

            cur.log(&pnt, Log::Error, msg, "");
            return;
        }

        self.validate_type(cur, exp, lookup);
    }
}
