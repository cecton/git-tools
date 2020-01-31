mod commands;
mod git;

use std::env;
use std::io::{self, Write};
use std::iter::Iterator;

use structopt::{StructOpt, clap::AppSettings};

#[derive(StructOpt, Debug)]
#[structopt(
    bin_name = "cargo git",
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub enum Opts {
    Add(Params),
    Checkout(Params),
    Commit(Params),
    Diff(Params),
}

#[derive(StructOpt, Debug)]
#[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
pub struct Params {
    args: Vec<String>,
}

impl Iterator for Params {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.args.is_empty() {
            None
        } else {
            Some(self.args.remove(0))
        }
    }
}

fn main() {
    let exit_status = execute();
    std::io::stdout().flush().unwrap();
    std::process::exit(exit_status);
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;

fn execute() -> i32 {
    // Drop extra `git` argument provided by `cargo`.
    let mut found_git = false;
    let args = env::args().filter(|x| {
        if found_git {
            true
        } else {
            found_git = x == "git";
            x != "git"
        }
    });

    let opts = Opts::from_iter(args);

    let res = match opts {
        Opts::Add(params) => commands::add::run(params),
        Opts::Commit(params) => commands::commit::run(params),
        Opts::Diff(params) => commands::diff::run(params),
        _ => todo!(),
    };

    if let Err(err) = res {
        eprintln!("{}", err);

        FAILURE
    } else {
        SUCCESS
    }
}
