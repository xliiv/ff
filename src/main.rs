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


// TODO:
// add ignore-dir to settings .git, etc.
// remove DirEntry() in msg "symlinked: .. -> .."
// add new readme
// release version

// adding(removing) dir? should walk over each file and `add` (`remove`) them
// bash autocompletion: https://kbknapp.github.io/clap-rs/clap/struct.App.html#examples-35
// make Config trait?
// TODO1: all action_ converts input to PathBuf and PathBuf is used so on


fn main() {
    run_cli();
}
