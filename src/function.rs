use crate::{
    block::Block,
    fields::Field,
    source::Source,
    typ::{Generic, Type},
};

#[allow(unused)]
#[derive(Debug)]
pub struct Function {
    pub name: String,
    pub arg: Vec<Field<Type>>,
    pub gen: Generic,
    pub ret: Type,
    pub block: Block,
}

impl Source {
    pub fn fun(&mut self) -> Function {
        let name = self.identifier();
        let de = self.expect_char(&['<', '(']);
        let gen = match de {
            '<' => self.gen(),
            _ => Vec::new(),
        };

        if de == '<' {
            self.expect_char(&['(']);
        }

        let arg = self.fields(')');

        self.expect(&["->"]);

        Function {
            name,
            arg,
            gen,
            ret: self.typ(),
            block: self.block(false),
        }
    }
}
