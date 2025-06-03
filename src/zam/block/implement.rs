use std::mem::swap;

use crate::{
    misc::{Bypass, Ref},
    parser::Parser,
    zam::block::BlockType,
};

impl Parser {
    pub fn implement(&mut self) -> Option<()> {
        let gen = match self.might('<') {
            true => self.dec_gen()?,
            _ => Default::default(),
        };
        let mut id_one = self.identifier(true, true)?;
        let trt_impl = self.expect(&["for", "{"])? == "for";
        let mut id_two = if trt_impl {
            self.identifier(true, true)?
        } else {
            Default::default()
        };

        if trt_impl {
            swap(&mut id_one, &mut id_two);
        }

        self.impls
            .bypass()
            .entry(Ref(&id_one.leaf_name().data))
            .or_default()
            .entry(Ref(self.log.path.as_path()))
            .or_default()
            .push(([id_one, id_two], gen, self.block(BlockType::Impl)?));

        Some(())
    }
}
