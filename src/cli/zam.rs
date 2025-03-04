use std::{collections::HashMap, path::PathBuf};

use crate::{
    cfg::{Config, PackageType},
    err,
    zam::{block::Hoistable, Zam},
};

pub fn zam(mut path: PathBuf, cfg: Config) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    let pkg = cfg.pkg.name;

    path.push("src");

    let mut stack = vec![path.clone()];
    let mut srcs = HashMap::new();
    let mut err = false;

    while let Some(v) = stack.pop() {
        let stamp = srcs.len();
        let mut tmp = Vec::new();
        let mut entries = v.read_dir().unwrap();

        while let Some(Ok(entry)) = entries.next() {
            let typ = entry.file_type().unwrap();
            let src = entry.path();

            if typ.is_dir() {
                tmp.push(src);
            } else if typ.is_file() && src.extension().is_some_and(|v| v == "z") {
                srcs.insert(
                    if !srcs.contains_key(&pkg) && entry.file_name() == "main.z" {
                        pkg.clone()
                    } else {
                        format!(
                            "{pkg}/{}",
                            src.strip_prefix(&path)
                                .unwrap()
                                .with_extension("")
                                .to_str()
                                .unwrap()
                        )
                    },
                    match Zam::parse(src) {
                        Some(v) => v,
                        _ => {
                            err = true;
                            continue;
                        }
                    },
                );
            }
        }

        if stamp != srcs.len() {
            stack.extend(tmp);
        }
    }

    let bin = cfg.pkg.typ.binary_search(&PackageType::Bin).is_ok();

    let src = srcs.get_mut(&pkg).unwrap();

    match src.block.dec.get("main") {
        Some(v) => match match **v {
            Hoistable::Function { .. } => false,
            Hoistable::Public(ref v) => match **v {
                Hoistable::Function { .. } => false,
                _ => true,
            },
            _ => true,
        } {
            false => {}
            true => src
                .parser
                .err_rng(v.rng, "identifier `main` exists but it's not a function"),
        },
        _ => src.parser.err_rng(
            [0, src.parser.data.len().checked_sub(1).unwrap_or_default()],
            "expected `main` function",
        ),
    };

    println!("{:?}", srcs.keys())
}
