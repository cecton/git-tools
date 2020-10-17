mod common;

use common::Git;

use std::env;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    bin_name = "git fork",
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub struct Fork {
    branch_name: String,
    from: Option<String>,
}

fn main() {
    let exit_status = execute();
    std::io::stdout().flush().unwrap();
    std::process::exit(exit_status);
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;

fn execute() -> i32 {
    let opts = Fork::from_args();

    if let Err(err) = run(opts) {
        eprintln!("{}", err);

        FAILURE
    } else {
        SUCCESS
    }
}

pub fn run(params: Fork) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    if git.has_file_changes()? {
        return Err("The repository has not committed changes, aborting.".into());
    }

    let branch_name = params.branch_name.as_str();
    let default_branch = git.get_default_branch("origin")?;
    let name = params
        .from
        .as_deref()
        .unwrap_or_else(|| default_branch.as_str());

    if name.contains('/') {
        git.update_upstream(name)?;
    }

    match git.get_branch_hash(name)? {
        // name is really a branch
        Some(hash) => git.branch(branch_name, Some(hash.as_str()))?,
        // name was not a branch
        None => git.branch(branch_name, Some(name))?,
    };

    git.switch_branch(branch_name)?;

    println!("Branch {} created.", branch_name);

    Ok(())
}
