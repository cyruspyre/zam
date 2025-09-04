#![feature(never_type, if_let_guard, unboxed_closures, fn_traits)]
#![deny(unused_must_use)]

mod analyzer;
mod cfg;
mod cli;
mod log;
mod misc;
mod naive_map;
mod parser;
mod perf;
mod zam;

fn main() {
    cli::start();
}
