use indexmap::IndexMap;

use crate::{
    log::{Log, Point},
    misc::Bypass,
};

use super::{expression::misc::Range, identifier::Identifier, Parser};

pub type Fields<T> = IndexMap<Identifier, T>;

pub trait FieldValue {
    fn field_value(src: &mut Parser) -> Option<Self>
    where
        Self: Sized;
}

impl Parser {
    pub fn fields<T: FieldValue>(&mut self, de: char) -> Option<Fields<T>> {
        self.ensure_closed(de)?;
        let mut fields: IndexMap<Identifier, T> = IndexMap::new();
        let mut dup = IndexMap::new();
        let log = self.log.bypass();

        loop {
            if self.might(de) {
                break;
            }

            let name = self.identifier(true, false)?;

            self.expect_char(&[':'])?;
            self.skip_whitespace();

            let two = self.idx + 1;
            let data = T::field_value(self)?;
            let src = &self.log.data;
            log.rng = [two, self.idx];

            while let Some(v) = src.get(log.rng[1]) {
                if v.is_ascii_whitespace() {
                    log.rng[1] -= 1
                } else {
                    break;
                }
            }

            if let Some((prev, _)) = fields.get_key_value(&name) {
                let rng = name.rng();

                dup.entry(name)
                    .or_insert(vec![(prev.rng(), Point::Error, "first declared here")])
                    .push((rng, Point::Error, ""))
            } else {
                fields.insert(name, data);
            }

            if self.expect_char(&[',', de])? == de {
                break;
            }
        }

        log.rng.fill(self.idx);

        for (k, mut v) in dup {
            log(
                &mut v,
                Log::Error,
                &format!("`{k}` declared multiple times"),
                "",
            );
        }

        self.de.pop_front();

        Some(fields)
    }
}
