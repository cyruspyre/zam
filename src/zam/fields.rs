use indexmap::IndexMap;

use crate::parser::{
    log::{Log, Point},
    span::{Identifier, Span},
};

use super::Parser;

pub type Fields<T> = IndexMap<Identifier, T>;

pub trait FieldValue {
    fn field_value(src: &mut Parser) -> Option<Self>
    where
        Self: Sized;
}

impl Parser {
    pub fn fields<T: FieldValue>(&mut self, de: char) -> Option<Fields<T>> {
        self.ensure_closed(de)?;
        let mut fields: IndexMap<Span<String>, T> = IndexMap::new();
        let mut dup = IndexMap::new();

        loop {
            if self.might(de) {
                break;
            }

            let name = self.identifier(true)?;

            self.expect_char(&[':'])?;
            self.skip_whitespace();

            let two = self.idx + 1;
            let data = T::field_value(self)?;
            self.rng = [two, self.idx];

            while let Some(v) = self.data.get(self.rng[1]) {
                if v.is_ascii_whitespace() {
                    self.rng[1] -= 1
                } else {
                    break;
                }
            }

            if let Some((prev, _)) = fields.get_key_value(&name) {
                dup.entry(name.data)
                    .or_insert(vec![(prev.rng, Point::Error, "first declared here")])
                    .push((name.rng, Point::Error, ""))
            } else {
                fields.insert(name, data);
            }

            if self.expect_char(&[',', de])? == de {
                break;
            }
        }

        self.rng.fill(self.idx);

        for (k, mut v) in dup {
            self.log(
                &mut v,
                Log::Error,
                &format!("`{k}` declared multiple times"),
                "",
            );
        }

        self.de.pop_back();

        Some(fields)
    }
}
