use crate::zam::identifier::Identifier;

use super::Parser;

#[derive(Debug, Clone, PartialEq)]
pub struct Trait {
    name: Identifier,
    sub: Vec<Trait>,
}

impl Parser {
    pub fn trt(&mut self) -> Option<Trait> {
        let name = self.identifier(true, false)?;
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
