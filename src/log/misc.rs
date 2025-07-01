use crate::{
    log::{Logger, Point},
    misc::CustomDrop,
};

impl Logger {
    pub fn ctx(
        &mut self,
        rng: [usize; 2],
        pnt: Point,
        label: &str,
    ) -> CustomDrop<impl FnMut() + use<'_>> {
        self.ctx = Some((rng, pnt, unsafe { &*(label as *const _) }));
        CustomDrop(|| self.ctx = None)
    }
}
