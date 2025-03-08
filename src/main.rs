#![feature(never_type)]
#![deny(unused_must_use)]

mod cfg;
mod cli;
mod parser;
mod validator;
mod zam;

use cli::start;

fn main() {
    start();
}
