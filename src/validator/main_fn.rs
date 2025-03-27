use strsim::jaro;

use crate::{
    parser::log::{Log, Point},
    zam::block::Hoistable,
};

use super::Validator;

impl Validator {
    pub fn main_fn(&mut self) -> Option<()> {
        let src = self.srcs.get_mut(&self.cfg.pkg.name)?;
        let v = match src
            .block
            .dec
            .iter()
            .map(|(k, v)| (jaro("main", k), k.rng, v))
            .max_by(|a, b| a.0.total_cmp(&b.0))
        {
            Some((sim, rng,  data)) => (
                rng,
                Point::Info,
                match data {
                    Hoistable::Function { .. } => match sim {
                        1.0 => return Some(()),
                        _ => "did you mean `main`?",
                    },
                    _ => match sim {
                        1.0 => "identifier `main` exists but isn't a function",
                        _ => "similar identifier as `main` exists",
                    },
                },
            ),
            _ => ([src.parser.data.len(), 0], Point::Error, ""),
        };

        src.parser.log(&[v], Log::Error, "expected `main` function");

        None
    }
}
