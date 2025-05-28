use std::path::PathBuf;

use crate::{cfg::Config, err, misc::Bypass, zam::Zam};

pub fn zam(mut path: PathBuf, cfg: Config) {
    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    path.push("src");

    let mut root = Zam::parse(path.join("main.z"), true);
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

            if typ.is_dir() {
                let zam = Zam::parse(path.join("mod.z"), false);
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

            mods.insert(key, Zam::parse(path, true));
        }

        // omit directories with empty zam src files
        if mods.len() == stamp
            && let Some(mut fun) = remover
        {
            fun()
        }
    }

    let mut tmp = vec![&mut root.mods];

    while let Some(v) = tmp.pop() {
        dbg!(v.keys());

        for (k, v) in v {
            let v = &mut v.mods;

            if !v.is_empty() {
                tmp.push(v);
            }
        }
    }

    // Project { cfg, srcs, impls }.validate(err)
}
