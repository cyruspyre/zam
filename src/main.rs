#![feature(never_type)]
#![feature(let_chains)]
#![feature(if_let_guard)]
#![feature(try_trait_v2)]
#![feature(iter_array_chunks)]
#![deny(unused_must_use)]

mod cfg;
mod cli;
mod misc;
mod parser;
mod validator;
mod zam;

use cli::start;

fn main() {
    start();
}
