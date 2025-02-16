mod cfg;
mod parser;
mod zam;

use std::{
    collections::HashMap,
    fs::{create_dir, create_dir_all, read_to_string, File},
    io::Write,
    path::PathBuf,
};

use cfg::{load, Config};
use clap::{arg, builder::PathBufValueParser, Command};

fn main() {
    let path = arg!([PATH] "Path to a file or a folder")
        .default_value(".")
        .value_parser(PathBufValueParser::new())
        .hide_default_value(true);
    let release = arg!(-r --release "Build and run in release mode");
    let mut shit = Command::new("zam")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("new").about("Create a new project").arg(&path),
            Command::new("run").about("Run a file or a project").args([
                &path,
                &release,
                &arg!(-a --aot "Build and run in AOT mode")
                    .default_value_if("release", "true", "true"),
            ]),
            Command::new("check")
                .arg(&path)
                .about("Analyze a file or a project"),
            Command::new("build")
                .about("Build a file or a project")
                .args([
                    path,
                    release,
                    arg!(--type <TYPE> "Type of output to produce")
                        .value_parser(["lib", "bin", "llvm-ir"]),
                ]),
        ])
        .get_matches();

    let (cmd, mut shit) = shit.remove_subcommand().unwrap();
    let mut path = shit
        .remove_one::<PathBuf>("PATH")
        .unwrap();
    let cfg = path.join("zam.toml");

    if cmd == "new" {
        if path.exists() {
            if !path.is_dir() {
                err!("expected a directory to initialize project")
            } else if path.read_dir().unwrap().next().is_some() {
                err!("directory not empty")
            }
        } else {
            create_dir_all(&path).unwrap()
        }

        File::create(&cfg).unwrap();
        File::create(path.join(".gitignore")).unwrap();
        path.push("src");
        create_dir(&path).unwrap();
        File::create(path.join("main.z"))
            .unwrap()
            .write(b"fn main() {\n    println(\"Hello world\")\n}")
            .unwrap();
        return;
    }

    path = path.canonicalize().unwrap();

    if !path.exists() {
        err!("path does not exist")
    }

    if !path.is_dir() {
        err!("path isn't a directory")
    }

    let cfg = load(path.join("zam.toml"));

    // path.pop();

    // println!("{path:?}");

    // let mut srcs = HashMap::new();
    // let mut stack = vec![path];

    // while let Some(path) = stack.pop() {
    //     for v in path.read_dir().unwrap() {
    //         let v = v.unwrap();
    //         let typ = v.file_type().unwrap();
    //         let path = v.path();

    //         if typ.is_dir() {
    //             stack.push(path);
    //         } else if path.extension().is_some_and(|v| v == "z") {
    //             srcs.insert(path.clone(), Parser::new(path));
    //         }
    //     }
    // }

    // println!("{:?}", srcs.keys())
}
