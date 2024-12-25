use std::{collections::BTreeMap, path::PathBuf, str::FromStr};

use clang::{Clang, Entity, EntityKind, Index, TypeKind};

use crate::{source::Source, typ::Type};

use super::{
    r#use::{Lib, Libs},
    External,
};

pub trait Prepare {
    fn prepare(self, src: &mut Source, ext: &mut Vec<External>);
}

impl Prepare for Libs {
    fn prepare(self, src: &mut Source, ext: &mut Vec<External>) {
        let path = &mut PathBuf::from("test.h");
        let clang = Clang::new().unwrap();
        let idx = Index::new(&clang, false, false);

        for Lib { name, rng, ids } in self {
            // path.push(name);
            if !path.exists() {
                src.err_rng(rng, "cannot find the file")
            }

            let tu = idx.parser(&path).parse().unwrap();
            let mut valid = tu
                .get_entity()
                .get_children()
                .into_iter()
                .filter_map(|v| match v.get_kind() {
                    EntityKind::FunctionDecl => Some((v.get_name().unwrap(), v)),
                    _ => None,
                })
                .collect::<BTreeMap<_, _>>();

            for (id, rng) in ids {
                if let Some(v) = valid.remove(&id) {
                    println!("{:?}", parse_c_fn(id, v));
                    todo!();
                    continue;
                }

                src.err_rng(rng, "unknown identifier");
            }
        }
    }
}

fn parse_c_fn(name: String, ent: Entity) {
    // add struct parsing to proceed further
    let ret = ent.get_result_type().unwrap().get_canonical_type();
    println!(
        "{:?} {} {}",
        ret.get_sizeof().unwrap() * 8,
        ret.is_signed_integer(),
        ret.is_integer()
    );
    match ret.get_kind() {
        _ => todo!("{:?}", ret),
    }
    println!("{:?}", ret);
}
