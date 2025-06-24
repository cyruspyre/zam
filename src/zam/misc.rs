use std::{
    borrow::Borrow,
    fmt::{Formatter, Result},
};

pub fn display<T: Borrow<String>>(val: &Vec<T>, f: &mut Formatter) -> Result {
    let mut buf = String::new();

    for (i, v) in val.iter().enumerate() {
        buf += v.borrow();

        let Some(v) = val.get(i + 1) else { break };
        let v: &str = v.borrow();

        if v == "" {
            buf += " as ";
            buf += v;
            buf += val[i + 2].borrow();
            break;
        }

        buf += "::"
    }

    f.write_str(&buf)
}
