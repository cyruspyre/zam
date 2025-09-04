use strsim::jaro;

use crate::{
    log::{Log, Point},
    zam::{Entity, expression::misc::Range},
};

use super::Project;

impl Project {
    pub fn main_fn(&mut self) {
        let zam = &mut self.root;
        let v = match zam
            .block
            .dec
            .iter()
            .map(|(k, v)| (jaro("main", k.leaf_name()), k.rng(), v))
            .max_by(|a, b| a.0.total_cmp(&b.0))
        {
            Some((sim, rng, data)) if sim > 0.8 => (
                rng,
                Point::Info,
                match data {
                    Entity::Function { .. } => match sim {
                        1.0 => return,
                        _ => "did you mean `main`?",
                    },
                    _ => match sim {
                        1.0 => "identifier `main` exists but isn't a function",
                        _ => "similar identifier as `main` exists",
                    },
                },
            ),
            _ => ([zam.log.data.len(); 2], Point::Error, ""),
        };

        zam.log
            .call(&mut [v], Log::Error, "expected `main` function", "");
    }
}
