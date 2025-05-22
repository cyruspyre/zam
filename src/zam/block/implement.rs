use std::mem::swap;

use crate::parser::Parser;

use super::{BlockType, Impls};

impl Parser {
    pub fn implement(&mut self, impls: &mut Impls) -> Option<()> {
        let gen = match self.might('<') {
            true => self.dec_gen()?,
            _ => Default::default(),
        };
        let mut id_one = self.identifier(true)?;
        let trt_impl = self.expect(&["for", "{"])? == "for";
        let mut id_two = if trt_impl {
            self.identifier(true)?
        } else {
            Default::default()
        };

        if trt_impl {
            swap(&mut id_one, &mut id_two);
        }

        impls
            .entry(id_one)
            .or_default()
            .entry(id_two)
            .or_default()
            .push((gen, self.block(BlockType::Impl)?));

        Some(())
    }
}
