use crate::zam::{
    expression::{term::Term, Expression},
    typ::kind::TypeKind,
};

use super::Validator;
/// 
/// 
/// 
/// TODO
/// 
/// IMPROVISE TYPE FOR INTEGER AND OTHER STUFF AS WELL
/// 
/// 
/// 
impl Validator {
    // At some point, you might think why this function is declared here instead of at `expression/mod.rs`.
    // Well no shit sherlock. Type inference requires context of the whole source.
    pub fn infer_typ(&mut self, exp: &mut Expression) -> Option<([usize; 2], &str)> {
        let mut typ = &exp.typ;
        let exp = &exp.data;

        if let Some(v) = exp.first() {
            if !v.valid_first_term() {
                return Some((v.rng, "expected a term beforehand"));
            }
        }

        for v in exp {
            let lol = match v.data {
                Term::Integer { bit, sign, .. } => {

                },
                // Term::Float { bit, .. } => format!("f{}", if bit == 0 { 32 } else { bit }),
                // Term::Integer { bit, sign, .. } => Lmao::Integer { bit: match bit {
                //     u32::MAX
                //     _ => {},
                // }, sign: () },
                _ => todo!(),
            };
        }

        None
    }
}
