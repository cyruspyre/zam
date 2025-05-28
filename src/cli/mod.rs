mod init;
mod zam;

use std::{num::NonZeroUsize, path::PathBuf};

use clap::{arg, builder::PathBufValueParser, value_parser, Command};
use init::init;
use zam::zam;

use crate::cfg::Config;

pub fn start() {
    let path = arg!([PATH] "Path to a file or a folder")
        .default_value(".")
        .value_parser(PathBufValueParser::new())
        .hide_default_value(true);
    let release = arg!(-r --release "Build and run in release mode");
    let jobs = arg!(-j --jobs <N> "Number of parallel jobs, defaults to # of CPUs")
        .value_parser(value_parser!(NonZeroUsize));
    let (name, mut cmd) = Command::new("zam")
        .disable_help_subcommand(true)
        .arg_required_else_help(true)
        .subcommands([
            Command::new("new")
                .about("Create a new project")
                .arg(&path)
                .args([
                    arg!(--bin "Use binary template")
                        .default_value("true")
                        .default_value_if("lib", "true", "false"),
                    arg!(--lib "Use library template"),
                ]),
            Command::new("run")
                .about("Run a file or a project")
                .args([&path, &release, &jobs])
                .arg(
                    arg!(-a --aot "Build and run in AOT mode")
                        .default_value_if("release", "true", "true"),
                ),
            Command::new("check")
                .about("Analyze a file or a project")
                .args([&path, &jobs]),
            Command::new("build")
                .about("Build a file or a project")
                .args([path, release, jobs]),
        ])
        .get_matches()
        .remove_subcommand()
        .unwrap();
    let path = cmd.remove_one::<PathBuf>("PATH").unwrap();
    let cfg = path.join("zam.toml");

    match name.as_str() {
        "new" => init(path, cfg, cmd),
        _ => zam(path, Config::load(cfg)),
    }
}
