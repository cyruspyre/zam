use crate::{
    log::Point,
    misc::Bypass,
    parser::Parser,
    zam::{block::BlockType, expression::misc::Range, identifier::Identifier, Entity},
};

impl Parser {
    pub fn r#trait<'a>(&mut self) -> Option<(Identifier, Entity)> {
        let log = self.log.bypass();
        let __ = self
            .bypass()
            .ctx(log.rng, Point::Info, "while parsing this trait");
        let name = self.identifier(true, false)?;

        log.ctx.unwrap().0 = name.rng();

        let gen = self.dec_gen()?;
        let item = self.block(BlockType::Trait)?.dec;

        Some((name, Entity::Trait { gen, item }))
    }
}
