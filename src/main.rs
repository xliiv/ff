//! See [Readme](https://github.com/xliiv/ff/blob/master/README.md) for use case

#![deny(missing_docs)]
#[macro_use]
extern crate clap;
extern crate ini;
extern crate tempdir;
extern crate walkdir;

pub mod cli;
pub mod config;
pub mod core;

use cli::*;


// TODO::
// functional test
// errors handling!!
// clippy
// keep trakcing-dir as ~/<path-tracking-dir> in cnofig file
// adding(removing) dir? should walk over each file and `add` (`remove`) them
// bash autocompletion: https://kbknapp.github.io/clap-rs/clap/struct.App.html#examples-35


fn main() {
    run_cli();
}