use super::Parser;

#[derive(Debug, Clone)]
pub struct Trait {
    name: String,
    sub: Vec<Trait>,
}

impl Parser {
    pub fn trt(&mut self) -> Trait {
        let name = self.identifier(false);
        let mut sub = Vec::new();

        if self.might('<') {
            loop {
                sub.push(self.trt());

                match self.expect_char(&[',', '>']) {
                    ',' => {}
                    _ => break,
                }
            }
        }

        Trait { name, sub }
    }
}
