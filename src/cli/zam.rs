use std::{ops::Deref, path::PathBuf};

use hashbrown::HashMap;
use threadpool::ThreadPool;

use crate::{
    analyzer::Project,
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    naive_map::NaiveMap,
    zam::{Zam, path::ZamPath},
};

pub fn zam(mut path: PathBuf, cfg: Config, pool: &ThreadPool) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    path.push("src");

    let mut zam_id = ZamPath(vec![Ref(&cfg.pkg.name)]);
    let mut impls = HashMap::new();
    let mut parse =
        |path: PathBuf, required: bool, id: ZamPath| Zam::parse(path, required, &mut impls, id);
    let mut root = parse(path.join("main.z"), true, zam_id.clone());
    let mut parent_mods = root.mods.bypass();
    let mut stack = Vec::from([(path.clone(), root.bypass())]);

    while let Some((cur, zam)) = stack.pop() {
        let mods = &mut zam.mods;
        let not_root = cur != path;
        let Ok(mut entries) = cur.read_dir() else {
            continue;
        };

        if not_root {
            zam_id.push(*zam.id.last().unwrap());
        }

        while let Some(res) = entries.next() {
            let Ok(entry) = res else { continue };
            let Ok(typ) = entry.file_type() else { continue };
            let entry_path = entry.path();
            let file = entry_path.with_extension("");
            let Some(name) = file.file_name() else {
                continue;
            };
            let Some(name) = name.to_str() else { continue };
            let name = name.to_string();

            if typ.is_dir() {
                let mut entry =
                    mods.entry(name)
                        .insert(parse(entry_path.join("mod.z"), false, zam_id.clone()));
                let val = entry.get_mut().bypass();

                val.id.push(Ref(entry.key()));
                stack.push((entry_path, val));

                continue;
            }

            if !typ.is_file() || !matches!(entry.path().extension(), Some(v) if v == "z") {
                continue;
            }

            let special = match name.as_str() {
                "main" if !not_root => continue,
                "mod" if not_root => continue,
                _ => true,
            };
            let mut entry = mods
                .entry(name)
                .insert(parse(entry_path, true, zam_id.clone()));

            if special {
                let tmp = Ref(entry.key());
                entry.get_mut().id.push(tmp);
            }
        }

        if not_root {
            zam_id.pop();

            if mods.is_empty() && zam.log.ignore {
                parent_mods.remove(zam.id.last().unwrap().deref());
            }

            parent_mods = mods;
        }
    }

    Project {
        cur: NaiveMap::new(),
        cfg,
        root,
        impls,
    }
    .validate();
}
