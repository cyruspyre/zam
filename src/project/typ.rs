use crate::{
    log::{Log, Point},
    project::Project,
    zam::typ::{Type, kind::TypeKind},
};

impl Project {
    pub fn assert_type(&mut self, actual: &mut Type, expected: &mut Type) -> Option<()> {
        let mut pnt = Vec::new();

        if let TypeKind::Unknown = expected.kind.data {
            *expected = actual.clone();
            return Some(());
        }

        if (actual.kind == expected.kind || coercible(&expected.kind, &actual.kind))
            && actual.ptr == expected.ptr
        {
            return Some(());
        }

        pnt.push((
            actual.kind.rng,
            Point::Error,
            format!("expected `{expected}`, found `{actual}`"),
        ));

        self.cur()
            .log
            .call(&mut pnt, Log::Error, "type mismatch", "");

        None
    }
}

fn coercible<'a>(a: &TypeKind, b: &TypeKind) -> bool {
    match a {
        TypeKind::Float(0) if matches!(b, TypeKind::Float(v) if *v != 0) => {}
        TypeKind::Integer {
            bit: 0,
            sign: sign_,
        } if matches!(b, TypeKind::Integer { bit, sign } if {
            *bit != 0 && sign == sign_ || *sign && !*sign_
        }) => {}
        _ => return false,
    }

    true
}
