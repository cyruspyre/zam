mod misc;

use std::{path::PathBuf, process::exit};

use serde::{de::Visitor, Deserialize, Deserializer};
use toml::de::Error;

use crate::parser::{misc::read_file, Parser};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "package")]
    pub pkg: Package,
    #[serde(skip, default = "misc::size")]
    pub bit: u32,
}

impl Config {
    pub fn load(path: PathBuf) -> Config {
        let data = read_file(&path);
        let toml: Config = match toml::from_str(&data) {
            Ok(v) => v,
            Err(err) => map_err(path, data, err),
        };

        toml
    }
}

#[derive(Debug, Deserialize)]
pub struct Package {
    pub name: String,
    #[serde(rename = "type", deserialize_with = "pkg_type")]
    pub typ: Vec<PackageType>,
}

fn pkg_type<'de, D>(de: D) -> Result<Vec<PackageType>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, SeqAccess};

    struct Tmp;

    impl<'de> Visitor<'de> for Tmp {
        type Value = Vec<PackageType>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("sequence of strings")
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut tmp = Vec::new();

            while let Some(value) = seq.next_element()? {
                tmp.push(value);
            }

            if tmp.is_empty() {
                return Err(Error::custom("expected at least one value"));
            }

            tmp.sort_unstable();

            Ok(tmp)
        }
    }

    de.deserialize_seq(Tmp)
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackageType {
    #[serde(rename = "bin")]
    Bin,
    #[serde(rename = "lib")]
    Lib,
}

fn map_err(path: PathBuf, data: String, err: Error) -> ! {
    let mut chars = Vec::with_capacity(data.len());
    let mut line = Vec::new();

    for (i, c) in data.chars().enumerate() {
        if c == '\n' {
            line.push(i);
        }

        chars.push(c);
    }
    let rng = err.span().unwrap();
    let mut rng = [rng.start, rng.end.checked_sub(1).unwrap_or_default()];

    if rng[0] > rng[1] {
        rng[0] -= 1;
        rng[1] = 0;
    }

    Parser {
        path,
        line,
        data: chars,
        ..Default::default()
    }
    .err_rng(rng, err.message());
    exit(1)
}
