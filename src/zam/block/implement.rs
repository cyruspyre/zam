use std::mem::swap;

use crate::{
    misc::{Bypass, Ref},
    parser::Parser,
    zam::block::{BlockType, LocalImpls},
};

impl Parser {
    pub fn implement(&mut self, impls: &mut LocalImpls, global: bool) -> Option<bool> {
        let generic = match self.might('<') {
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

        let tmp = if global {
            self.impls
                .bypass()
                .entry(Ref(&id_one.leaf_name().data))
                .or_default()
                .entry(self.id)
                .or_default()
        } else {
            impls.entry(Ref(&id_one.leaf_name().data)).or_default()
        };

        tmp.push(([id_one, id_two], generic, self.block(BlockType::Impl)?));

        Some(true)
    }
}
