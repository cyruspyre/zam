use std::path::PathBuf;

use serde::Deserialize;
use toml::de::Error;

use crate::parser::{misc::read_file, Parser};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(rename = "package")]
    pkg: Package,
}

#[derive(Debug, Deserialize)]
pub struct Package {
    name: String,
}

pub fn load(path: PathBuf) -> Config {
    let data = read_file(&path);
    let toml: Config = match toml::from_str(&data) {
        Ok(v) => v,
        Err(err) => map_err(path, data, err),
    };

    todo!()
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
    .err_rng(rng, err.message())
}
