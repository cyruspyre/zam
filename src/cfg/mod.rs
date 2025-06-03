mod misc;

use std::{
    fmt::{self, Formatter},
    fs::read_to_string,
    path::PathBuf,
    process::exit,
};

use indexmap::IndexMap;
use semver::VersionReq;
use serde::{
    de::{Error, IntoDeserializer, MapAccess, Visitor},
    Deserialize, Deserializer,
};

use crate::{err, log::Logger};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    #[serde(rename = "package")]
    pub pkg: Package,
    #[serde(rename = "dependencies", deserialize_with = "deps")]
    pub deps: IndexMap<String, VersionReq>,
    #[serde(skip, default = "misc::size")]
    pub bit: u32,
}

impl Config {
    pub fn load(path: PathBuf) -> Config {
        let Ok(data) = read_to_string(&path) else {
            err!(
                "couldn't find `{}` in `{}`",
                path.file_name().unwrap().to_string_lossy(),
                path.parent().unwrap().display()
            )
        };
        let toml: Config = match toml::from_str(&data) {
            Ok(v) => v,
            Err(err) => map_err(path, data, err),
        };

        toml
    }
}

fn deps<'de, D>(de: D) -> Result<IndexMap<String, VersionReq>, D::Error>
where
    D: Deserializer<'de>,
{
    struct _Visitor;

    impl<'de> Visitor<'de> for _Visitor {
        type Value = IndexMap<String, VersionReq>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("table")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut tmp = IndexMap::new();

            while let Some((key, value)) = map.next_entry::<String, _>()? {
                if let Some(e) = validate_id(&key) {
                    return Err(e);
                }

                if tmp.insert(key, value).is_some() {
                    return Err(Error::duplicate_field(tmp.pop().unwrap().0.leak()));
                }
            }

            Ok(tmp)
        }
    }

    de.deserialize_map(_Visitor)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Package {
    #[serde(deserialize_with = "pkg_name")]
    pub name: String,
    #[serde(rename = "type", deserialize_with = "pkg_type")]
    pub typ: Vec<PackageType>,
}

fn validate_id<E: Error>(v: &str) -> Option<E> {
    let mut iter = v.chars();
    let Some(first) = iter.next() else {
        return Some(Error::custom("empty identifier"));
    };

    None
}

fn pkg_name<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    struct _Visitor;

    impl<'de> Visitor<'de> for _Visitor {
        type Value = String;

        fn expecting(&self, f: &mut Formatter) -> fmt::Result {
            f.write_str("valid identifier")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match validate_id(&v) {
                Some(e) => Err(e),
                _ => Ok(v.to_string()),
            }
        }
    }

    Ok(deserializer.deserialize_str(_Visitor)?)
}

fn pkg_type<'de, D>(de: D) -> Result<Vec<PackageType>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{Error, SeqAccess};

    struct Tmp;

    impl<'de> Visitor<'de> for Tmp {
        type Value = Vec<PackageType>;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            formatter.write_str("sequence of strings")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
        {
            Ok(vec![PackageType::deserialize(v.into_deserializer())?])
        }

        fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
            let mut tmp = Vec::with_capacity(seq.size_hint().unwrap_or_default());
            let mut dup = None;

            while let Some(value) = seq.next_element()? {
                if tmp.contains(&value) {
                    dup = Some(value);
                    break;
                }

                tmp.push(value)
            }

            if tmp.is_empty() {
                return Err(Error::custom("expected at least one value"));
            }

            if let Some(v) = dup {
                return Err(Error::custom(format!(
                    "duplicate element `{}`",
                    match v {
                        PackageType::Bin => "bin",
                        PackageType::Lib => "lib",
                    }
                )));
            }

            tmp.sort_unstable();

            Ok(tmp)
        }
    }

    de.deserialize_any(Tmp)
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum PackageType {
    Bin,
    Lib,
}

fn map_err(path: PathBuf, data: String, err: toml::de::Error) -> ! {
    let mut chars = Vec::with_capacity(data.len());
    let mut line = Vec::new();

    for (i, c) in data.chars().enumerate() {
        if c == '\n' {
            line.push(i);
        }

        chars.push(c);
    }

    let rng = err.span().unwrap();
    let mut rng = [rng.start, rng.end];

    if rng[0] != rng[1] {
        rng[1] -= 1
    }

    Logger {
        path,
        line,
        data: chars,
        ..Default::default()
    }
    .err_rng(rng, err.message());
    exit(1)
}
