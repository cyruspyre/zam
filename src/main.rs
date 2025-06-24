#![feature(never_type, let_chains, if_let_guard, unboxed_closures, fn_traits)]
#![deny(unused_must_use)]

mod cfg;
mod cli;
mod log;
mod misc;
mod naive_map;
mod parser;
mod perf;
mod project;
mod zam;

fn main() {
    cli::start();
}
