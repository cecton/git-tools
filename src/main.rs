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
    /// Same as `git add` but ignores Cargo.lock
    Add(Params),
    /// Check that HEAD can be merged without conflict
    ///
    /// If the `revision` argument is not provided, the parent branch is used. If the parent branch
    /// is missing, origin/master is used.
    Check(Check),
    /// Same as `git checkout` but ignores Cargo.lock
    ///
    /// This is the equivalent of `git checkout [params] -- $(git diff --name-only | grep -v Cargo.lock)`
    Checkout(Params),
    /// Commit the files staged with the message WIP and add the parent branch to the commit
    /// message
    Commit(Commit),
    /// Delete an existing branch locally and remotely (its upstream)
    Delete(Delete),
    /// Same as `git diff` but ignores Cargo.lock
    Diff(Params),
    /// Create a new branch (based on origin/master by default) and switch to it. Also make an init
    /// commit to track the forking branch (parent branch) and commit it came from.
    Fork(Fork),
    /// Merge branch to the current branch with a merge commit only (traditional merge) and delete
    /// the local and remote branch afterwards
    ///
    /// This command will fail if there is any conflict. (If the branch given is not up-to-date
    /// enough with the current branch.)
    Merge(Merge),
    /// Try to push the current branch to its remote branch. If the remote branch does not exist,
    /// create one with the same name and set it on the local branch.
    Push(Params),
    /// This command allows squashing the current branch by reseting to the parent branch. This
    /// command should be followed ultimately by git commit.
    ///
    /// This command will fail if the current branch is not up-to-date with the parent.
    Squash(Squash),
    /// Update the current branch by merging the parent branch to the current branch.
    ///
    /// This command merges the missing commits from the base branch to the current branch and
    /// stops right before it encounters a conflict.
    ///
    /// If the first commit to merge is conflicting, it does a merge alone of this commit, allowing
    /// the user to resolve it and commit.
    ///
    /// The command can be (is intended to be) repeated until the current branch has no missing
    /// commit.
    ///
    /// It ignores Cargo.lock conflicts by taking the Cargo.lock of the current branch.
    ///
    /// The option `--deps` can be used to finally update the Cargo.lock in its own commit.
    Update(Update),
}

#[derive(StructOpt, Debug)]
#[structopt(settings = &[AppSettings::TrailingVarArg, AppSettings::AllowLeadingHyphen])]
pub struct Params {
    args: Vec<String>,
}

#[derive(StructOpt, Debug)]
pub struct Check {
    /// Revision to check conflict with (parent branch by default or origin/master)
    revision: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Commit {
    #[structopt(long, short = "m", default_value = "WIP")]
    message: String,

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
    #[structopt(long)]
    deps: bool,

    // NOTE: the long and short name for the parameters must not conflict with `git merge`
    /// Do not run `git merge` at the end. (Merge to the latest commit possible without conflict.)
    #[structopt(long, short = "u")]
    no_merge: bool,

    /// Use a specific revision instead of parent branch to update.
    #[structopt(long, short = "r")]
    revision: Option<String>,

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
        Opts::Check(params) => commands::check::run(params),
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
