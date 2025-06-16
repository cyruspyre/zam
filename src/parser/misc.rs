use crate::{log::Point, misc::CustomDrop, parser::Parser};

pub trait CharExt {
    fn is_id(&self) -> bool;
    fn is_quote(&self) -> bool;
}

impl CharExt for char {
    fn is_id(&self) -> bool {
        *self == '_' || self.is_ascii_alphanumeric()
    }

    fn is_quote(&self) -> bool {
        matches!(self, '"' | '\'')
    }
}

impl Parser {
    pub fn ctx(
        &mut self,
        rng: [usize; 2],
        pnt: Point,
        label: &str,
    ) -> CustomDrop<impl FnMut() + use<'_>> {
        self.log.ctx = Some((rng, pnt, unsafe { &*(label as *const _) }));
        CustomDrop(|| self.log.ctx = None)
    }
}
