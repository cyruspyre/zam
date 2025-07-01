mod init;
mod zam;

use std::{num::NonZero, path::PathBuf, thread::available_parallelism};

use clap::{Command, arg, builder::PathBufValueParser, value_parser};
use init::init;
use threadpool::ThreadPool;
use zam::zam;

use crate::cfg::Config;

pub fn start() {
    let path = arg!([PATH] "Path to a file or a folder")
        .default_value(".")
        .value_parser(PathBufValueParser::new())
        .hide_default_value(true);
    let release = arg!(-r --release "Build and run in release mode");
    let jobs = arg!(-j --jobs <N> "Number of parallel jobs, defaults to # of CPUs")
        .value_parser(value_parser!(NonZero<usize>));
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

    if name == "new" {
        return init(path, cfg, cmd);
    }

    let num_threads = if let Some(v) = cmd.remove_one("jobs") {
        v
    } else if let Ok(v) = available_parallelism() {
        v
    } else {
        NonZero::new(1).unwrap()
    };
    let pool = ThreadPool::new(num_threads.get());

    zam(path, Config::load(cfg), &pool);
    pool.join();
}
