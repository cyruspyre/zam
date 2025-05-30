use std::path::PathBuf;

use indexmap::IndexMap;
use threadpool::ThreadPool;

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    project::Project,
    zam::{identifier::Identifier, Zam},
};

pub fn zam(mut path: PathBuf, cfg: Config, pool: &ThreadPool) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    path.push("src");

    let mut impls: IndexMap<Identifier, IndexMap<_, _>> = IndexMap::new();
    let mut parse = |path: PathBuf, required: bool, id: Ref<String>| {
        let mut zam = Zam::parse(path, required);

        zam.id = id;

        while let Some((key, v)) = zam.block.impls.pop() {
            impls
                .entry(key)
                .or_default()
                .entry(Ref(zam.parser.path.as_path()))
                .or_insert(v);
        }

        zam
    };
    let mut root = parse(path.join("main.z"), true, Ref(&cfg.pkg.name));
    let mut stack = Vec::from([(path, &mut root.mods, None)]);

    while let Some((cur, mods, remover)) = stack.pop() {
        let Ok(mut entries) = cur.read_dir() else {
            continue;
        };
        let stamp = mods.len();

        while let Some(res) = entries.next() {
            let Ok(entry) = res else { continue };
            let Ok(typ) = entry.file_type() else { continue };
            let path = entry.path();
            let key = match path.with_extension("").file_name() {
                Some(v) if let Some(v) = v.to_str() => v.to_string(),
                _ => continue,
            };
            let id = Ref(&key);

            if typ.is_dir() {
                let zam = parse(path.join("mod.z"), false, id);
                let mut entry = mods.bypass().entry(key).insert_entry(zam);
                let parent = mods.bypass();

                stack.push((
                    path,
                    entry.get_mut().mods.bypass(),
                    Some(move || {
                        parent.swap_remove(entry.key());
                    }),
                ));
                continue;
            }

            if !typ.is_file()
                || *match entry.file_name().to_str() {
                    Some("main.z") => &path,
                    Some("mod.z") => &root.parser.path,
                    _ => &cur,
                } == path
            {
                continue;
            }

            mods.insert(key, parse(path, true, id));
        }

        // omit directories with empty zam src files
        if mods.len() == stamp
            && let Some(mut fun) = remover
        {
            fun()
        }
    }

    Project { cfg, root, impls }.validate();
}
