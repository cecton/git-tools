mod commands;
mod git;

use std::env;
use std::io::Write;
use std::iter::Iterator;

use structopt::{clap::AppSettings, StructOpt};

#[derive(StructOpt, Debug)]
#[structopt(
    bin_name = "cargo git",
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub enum Opts {
    Add(Params),
    Checkout(Params),
    Commit(Params),
    Delete(Delete),
    Diff(Params),
    Fork(Fork),
    Merge(Merge),
    Push(Params),
    Squash(Squash),
    Update(Update),
}

#[derive(StructOpt, Debug)]
#[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
pub struct Params {
    args: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Delete {
    branch_name: String,
}

#[derive(StructOpt, Debug)]
pub struct Fork {
    branch_name: String,
    #[structopt(default_value = "origin/master")]
    from: String,
}

#[derive(StructOpt, Debug)]
pub struct Merge {
    branch_name: String,
}

#[derive(StructOpt, Debug)]
pub struct Squash {
    /// Revision to move to (fork point by default)
    revision: Option<String>,
}

#[derive(StructOpt, Debug)]
#[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
pub struct Update {
    /// Runs cargo update and commit only Cargo.lock alone
    #[structopt(long = "deps")]
    deps: bool,

    // NOTE: the long and short name for the parameters must not conflict with `git merge`
    /// Do not run `git merge` at the end. (Merge to the latest commit possible without conflict.)
    #[structopt(long = "no-merge", short = "u")]
    no_merge: bool,

    merge_args: Vec<String>,
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
        Opts::Checkout(params) => commands::checkout::run(params),
        Opts::Commit(params) => commands::commit::run(params),
        Opts::Delete(params) => commands::delete::run(params),
        Opts::Diff(params) => commands::diff::run(params),
        Opts::Fork(params) => commands::fork::run(params),
        Opts::Merge(params) => commands::merge::run(params),
        Opts::Push(params) => commands::push::run(params),
        Opts::Squash(params) => commands::squash::run(params),
        Opts::Update(params) => commands::update::run(params),
    };

    if let Err(err) = res {
        eprintln!("{}", err);

        FAILURE
    } else {
        SUCCESS
    }
}
