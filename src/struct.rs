use crate::{
    fields::Field,
    source::Source,
    typ::{Generic, Type},
};

#[derive(Debug, Clone)]
pub struct Struct {
    pub name: String,
    pub gen: Generic,
    pub fields: Vec<Field<Type>>,
    pub rng: [usize; 2],
}

impl Source {
    pub fn strukt(&mut self) -> Struct {
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

        Struct {
            name,
            gen,
            fields: self.fields('}'),
            rng,
        }
    }
}
