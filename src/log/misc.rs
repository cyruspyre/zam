use std::borrow::Cow;

use crate::{
    log::{Logger, Point},
    misc::{Bypass, CustomDrop},
};

impl Logger {
    pub fn ctx(
        &mut self,
        rng: [usize; 2],
        pnt: Point,
        label: Cow<'static, str>,
    ) -> CustomDrop<impl FnMut() + use<>> {
        let tmp = self.ctx.bypass();
        *tmp = Some((rng, pnt, label));
        CustomDrop(|| *tmp = None)
    }
}
