use std::path::PathBuf;

use hashbrown::HashMap;
use threadpool::ThreadPool;

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    project::Project,
    zam::{path::ZamPath, Zam},
};

pub fn zam(mut path: PathBuf, cfg: Config, pool: &ThreadPool) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    path.push("src");

    let zam_id = ZamPath(vec![Ref(&cfg.pkg.name)]);
    let mut impls = HashMap::new();
    let mut parse =
        |path: PathBuf, required: bool, id: ZamPath| Zam::parse(path, required, &mut impls, id);
    let mut root = parse(path.join("main.z"), true, zam_id.clone());
    let mut stack = Vec::from([(path, zam_id, &mut root.mods, None::<Box<dyn FnOnce()>>)]);

    while let Some((cur, mut zam_id, mods, remover)) = stack.pop() {
        let Ok(mut entries) = cur.read_dir() else {
            continue;
        };
        let mut val = entries.next();

        if val.is_none()
            && let Some(fun) = remover
        {
            fun()
        }

        while let Some(res) = val.take() {
            val = entries.next();

            let Ok(entry) = res else { continue };
            let Ok(typ) = entry.file_type() else { continue };
            let path = entry.path();
            let key = match path.with_extension("").file_name() {
                Some(v) if let Some(v) = v.to_str() => v.to_string(),
                _ => continue,
            };

            if typ.is_dir() {
                zam_id.push(Ref(&key));
                let zam = parse(path.join("mod.z"), false, zam_id.clone());
                let mut entry = mods.bypass().entry(key).insert(zam);
                let parent = mods.bypass();

                stack.push((
                    path,
                    zam_id.clone(),
                    entry.get_mut().mods.bypass(),
                    Some(Box::new(move || {
                        parent.remove(entry.key());
                    })),
                ));
                zam_id.pop();
                continue;
            }

            if !typ.is_file() || {
                let tmp = cur == root.log.path.parent().unwrap();

                if key == "main" {
                    tmp
                } else {
                    !tmp
                }
            } {
                continue;
            }

            zam_id.push(Ref(&key));
            mods.insert(key, parse(path, true, zam_id.clone()));
            zam_id.pop();
        }
    }

    Project {
        cur: Vec::new(),
        cfg,
        root,
        impls,
    }
    .validate();
}
