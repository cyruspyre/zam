use super::{Hoistable, Parser};

impl Parser {
    pub fn strukt(&mut self) -> (String, Hoistable) {
        let name = self.identifier(false);
        let rng = self.rng;
        let de = self.expect_char(&['<', '{']);
        let gen = match de {
            '<' => self.gen(),
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['{']);
        }

        (
            name,
            Hoistable::Struct {
                gen,
                fields: self.fields('}'),
                rng,
            },
        )
    }
}
