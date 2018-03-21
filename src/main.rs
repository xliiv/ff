//! See [Readme](https://github.com/xliiv/ff/blob/master/README.md) for use case

#![deny(missing_docs)]
#[macro_use]
extern crate clap;
extern crate fui;
extern crate ini;
extern crate tempdir;
extern crate walkdir;

pub mod cli;
pub mod config;
pub mod core;

use cli::*;

fn main() {
    run_cli();
}
