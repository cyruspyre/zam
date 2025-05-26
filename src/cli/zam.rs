use std::path::PathBuf;

use indexmap::IndexMap;

use crate::{cfg::Config, err, misc::Ref, validator::Project, zam::Zam};

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
    let mut srcs = IndexMap::new();
    let mut impls = IndexMap::new();
    let mut err = 0;

    while let Some(v) = stack.pop() {
        let stamp = srcs.len();
        let mut tmp = Vec::new();
        let mut entries = v.read_dir().unwrap();

        'tmp: while let Some(Ok(entry)) = entries.next() {
            let typ = entry.file_type().unwrap();
            let src = entry.path();

            if typ.is_dir() {
                tmp.push(src);
                continue;
            } else if !typ.is_file() || !src.extension().is_some_and(|v| v == "z") {
                continue;
            }

            let mut key = pkg.clone();

            if entry.file_name() != "main.z" || path != src.parent().unwrap() {
                let mut tmp = src.strip_prefix(&path).unwrap().with_extension("");

                if entry.file_name() == "mod.z" {
                    tmp.pop();
                }

                for v in &tmp {
                    let Some(v) = v.to_str() else {
                        continue 'tmp;
                    };

                    key.push(v.to_string());
                }
            }

            let mut value = match Zam::parse(src) {
                Some(v) => v,
                _ => {
                    err += 1;
                    continue;
                }
            };

            impls.insert(Ref(&key), value.block.impls.take().unwrap());
            srcs.insert(key, value);
        }

        if stamp != srcs.len() {
            stack.extend(tmp);
        }
    }

    Project { cfg, srcs, impls }.validate(err)
}
