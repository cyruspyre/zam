use strsim::jaro;

use crate::{
    misc::Bypass,
    parser::log::{Log, Point},
    zam::{block::Hoistable, typ::kind::TypeKind},
};

use super::Validator;

impl Validator {
    pub fn identifier(&mut self) {
        for src in self.bypass().srcs.values_mut() {
            let dec = src.block.dec.bypass();
            let mut stack = vec![dec.bypass()];

            for (id, v) in dec {
                let Some((rng, msg)) = (match v {
                    Hoistable::Variable { val: exp, .. } => {
                        let kind = exp.typ.kind.bypass();
                        let rng = kind.rng;
                        let label = kind.bypass().try_as_number(self.cfg.bit);

                        'tmp: {
                            let TypeKind::ID(id) = &kind.data else {
                                break 'tmp;
                            };
                            let mut pnt = Vec::new();
                            let msg = match stack.iter().rev().find_map(|v: _| v.get_key_value(id))
                            {
                                Some((id, v)) => {
                                    let kind = match v {
                                        Hoistable::Variable { .. } => "variable",
                                        Hoistable::Function { .. } => "function",
                                        _ => break 'tmp,
                                    };

                                    pnt.push((id.rng, Point::Info, format!("{kind} defined here")));

                                    format!("expected type, found {kind} `{id}`",)
                                }
                                _ => format!("cannot find type `{id}`"),
                            };

                            let a = stack
                                .iter()
                                .map(|v| v.keys())
                                .flatten()
                                .max_by(|a, b| jaro(a, id).total_cmp(&jaro(b, id)))
                                .unwrap();
                            let b = [a.as_str(), "isize", "usize"]
                                .map(|v| (jaro(v, id), v))
                                .into_iter()
                                .max_by(|a, b| jaro(a.1, id).total_cmp(&jaro(b.1, id)))
                                .unwrap();
                            let label = match label {
                                Some(v) => v,
                                _ if b.0 >= 0.8 => {
                                    pnt.push((a.rng, Point::Info, "defined here".into()));
                                    format!("did you mean `{}`?", b.1)
                                }
                                _ => "not a type".into(),
                            };

                            pnt.push((rng, Point::Error, label));
                            src.parser.log(&pnt, Log::Error, msg);
                        }

                        //exp.data[0].as_type(&exp.typ, &self.cfg);
                        // dbg!(&exp.data);

                        println!("{kind:?}");

                        self.infer_typ(exp)
                    }
                    _ => None,
                }) else {
                    continue;
                };

                src.parser.err_rng(rng, msg);
            }
        }
    }
}
