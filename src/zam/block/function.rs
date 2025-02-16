use super::{Hoistable, Parser};

impl Parser {
    pub fn fun(&mut self) -> (String, Hoistable) {
        let name = self.identifier(false);
        let de = self.expect_char(&['<', '(']);
        let gen = match de {
            '<' => self.gen(),
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['(']);
        }

        let arg = self.fields(')');
        println!("{arg:?}");

        self.expect(&["->"]);

        (
            name,
            Hoistable::Function {
                arg,
                gen,
                ret: self.typ(),
                block: Some(self.block(false)),
            },
        )
    }
}
