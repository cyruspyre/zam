mod block;
mod fun;
pub mod lookup;
mod main_fn;
mod r#struct;
mod typ;
mod variable;

use std::{
    fmt::{Debug, Display},
    ops::{Deref, DerefMut},
};

use indexmap::IndexMap;
use lookup::{Current, Lookup};
use serde::{Deserialize, Deserializer};

use crate::{
    cfg::Config,
    err,
    misc::{Bypass, Ref},
    zam::{block::Impls, Zam},
};

#[derive(Clone, Hash, PartialEq, Eq)]
pub struct ZamID(Vec<String>);

impl<'a> Deserialize<'a> for ZamID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>,
    {
        Ok(Self::from(vec![String::deserialize(deserializer)?]))
    }
}

impl Deref for ZamID {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ZamID {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<Vec<String>> for ZamID {
    fn from(value: Vec<String>) -> Self {
        Self(value)
    }
}

impl Debug for ZamID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.to_string())
    }
}

impl Display for ZamID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.join("::"))
    }
}

pub struct Project {
    pub cfg: Config,
    pub srcs: IndexMap<ZamID, Zam>,
    pub impls: IndexMap<Ref<ZamID>, Impls>,
}

impl Project {
    pub fn validate(mut self, mut err: usize) {
        self.main_fn();

        for (id, src) in &mut self.bypass().srcs {
            let mut lookup = Lookup {
                validator: self.bypass(),
                cur: Current {
                    id,
                    parser: &mut src.parser,
                },
                var: IndexMap::new(),
                stack: Vec::new(),
            };

            self.block(&mut src.block, &mut lookup);
            err += lookup.cur.parser.err;
        }

        if err != 0 {
            err!(
                "could not compile `{}` due to {err} previous error{}",
                self.cfg.pkg.name,
                match err {
                    1 => "",
                    _ => "s",
                }
            )
        }
    }
}
