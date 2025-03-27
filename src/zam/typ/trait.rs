use crate::parser::span::Identifier;

use super::Parser;

#[derive(Debug, Clone)]
pub struct Trait {
    name: Identifier,
    sub: Vec<Trait>,
}

impl Parser {
    pub fn trt(&mut self) -> Option<Trait> {
        let name = self.identifier(true)?;
        let mut sub = Vec::new();

        if self.might('<') {
            loop {
                sub.push(self.trt()?);

                match self.expect_char(&[',', '>'])? {
                    ',' => {}
                    _ => break,
                }
            }
        }

        Some(Trait { name, sub })
    }
}
