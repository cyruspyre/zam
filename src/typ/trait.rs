use crate::source::Source;

#[derive(Debug, Clone)]

pub struct Trait {
    name: String,
    sub: Vec<Trait>,
}

impl Source {
    pub fn trt(&mut self) -> Trait {
        let name = self.identifier();
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
