use std::collections::HashMap;

use super::Parser;

#[derive(Debug, Clone)]
pub struct Field<T> {
    pub name: String,
    pub data: T,
    pub rng: [[usize; 2]; 2],
}

impl Parser {
    pub fn fields<T: FieldValue>(&mut self, de: char) -> Vec<Field<T>> {
        self.ensure_closed(de);
        let mut fields = Vec::new();

        loop {
            let name = self.identifier(true);
            let one = self.rng;

            if name.is_empty() {
                if let Some(c) = self._next() {
                    if c == de {
                        break;
                    }
                }

                self.err_op(false, &["<identifier>"])
            }

            self.expect_char(&[':']);
            self.skip_whitespace();
            let two = self.idx + 1;
            let data = T::field_value(self);
            self.rng = [two, self.idx];

            while let Some(v) = self.data.get(self.rng[1]) {
                if v.is_ascii_whitespace() {
                    self.rng[1] -= 1
                } else {
                    break;
                }
            }

            fields.push(Field {
                name,
                data,
                rng: [one, [two, self.idx]],
            });

            if self.might(de) {
                break;
            }

            self.expect_char(&[',']);
        }

        self.rng.fill(self.idx);

        let mut tmp: HashMap<_, Vec<_>> = HashMap::with_capacity(fields.len());

        for v in &fields {
            tmp.entry(&v.name).or_default().push(v.rng[0]);
        }

        for v in tmp.values_mut() {
            if v.len() < 2 {
                continue;
            }

            self.err_mul(v, "declared multiple times")
        }

        self.de.pop_back();

        fields
    }
}

pub trait FieldValue {
    fn field_value(src: &mut Parser) -> Self;
}
