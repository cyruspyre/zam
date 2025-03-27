use std::{collections::HashMap, path::PathBuf};

use crate::{cfg::Config, err, validator::Validator, zam::Zam};

pub fn zam(mut path: PathBuf, cfg: Config) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    let pkg = &cfg.pkg.name;

    path.push("src");

    let mut stack = vec![path.clone()];
    let mut srcs = HashMap::new();
    let mut err = 0;

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
                    if !srcs.contains_key(pkg) && entry.file_name() == "main.z" {
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
                            err += 1;
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

    Validator::new(cfg, srcs).validate(err);
}
