use std::{
    fs::{create_dir, create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use clap::ArgMatches;

use crate::err;

pub fn init(mut path: PathBuf, cfg: PathBuf, cmd: ArgMatches) {
    if path.exists() {
        if !path.is_dir() {
            err!("expected a directory to initialize project")
        } else if path.read_dir().unwrap().next().is_some() {
            err!("directory not empty")
        }
    } else {
        create_dir_all(&path).unwrap()
    }

    let lib = cmd.get_flag("lib");
    let mut typ = Vec::with_capacity(2);
    let mut src = String::new();

    if lib {
        typ.push("lib");
        src += "pub fn sum(a: u7, b: u7) -> u7 {\n    a + b\n}";
    }

    if cmd.get_flag("bin") {
        typ.push("bin");
        src += &format!(
            "{}fn main() -> _ {{\n    println(\"Hello World!{}\")\n}}",
            match lib {
                true => "\n\n",
                _ => "",
            },
            match lib {
                true => " {sum(43, 34)}",
                _ => "",
            }
        );
    }

    File::create(&cfg)
        .unwrap()
        .write(
            format!(
                "[package]\nname = \"{}\"\ntype = {typ:?}",
                path.file_name().unwrap().to_string_lossy(),
            )
            .as_bytes(),
        )
        .unwrap();
    File::create(path.join(".gitignore")).unwrap();

    path.push("src");
    create_dir(&path).unwrap();

    File::create(path.join("main.z"))
        .unwrap()
        .write(src.as_bytes())
        .unwrap();
}
