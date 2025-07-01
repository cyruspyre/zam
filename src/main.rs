#![feature(never_type, let_chains, if_let_guard, unboxed_closures, fn_traits)]
#![deny(unused_must_use)]
#![feature(btree_entry_insert)]

mod cfg;
mod cli;
mod log;
mod misc;
mod naive_map;
mod parser;
mod perf;
mod project;
mod zam;

struct Lol {
    a: u8,
    b: usize,
}

fn main() {
    cli::start();
}
