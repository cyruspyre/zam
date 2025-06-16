use std::{
    borrow::Borrow,
    fmt::{Formatter, Result},
};

pub fn display<T: Borrow<String>>(val: &Vec<T>, f: &mut Formatter) -> Result {
    let mut buf = String::new();

    for i in 0..val.len().checked_sub(1).unwrap_or_default() {
        buf += val[i].borrow();
        buf += "::";
    }

    if let Some(v) = val.last() {
        buf += v.borrow()
    }

    f.write_str(&buf)
}
