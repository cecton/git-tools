mod common;

use common::Git;

use std::env;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    bin_name = "git delete",
    about = env!("CARGO_PKG_DESCRIPTION")
)]
pub struct Delete {
    branch_name: String,
}

fn main() {
    let exit_status = execute();
    std::io::stdout().flush().unwrap();
    std::process::exit(exit_status);
}

const SUCCESS: i32 = 0;
const FAILURE: i32 = 1;

fn execute() -> i32 {
    let opts = Delete::from_args();

    if let Err(err) = run(opts) {
        eprintln!("{}", err);

        FAILURE
    } else {
        SUCCESS
    }
}

pub fn run(params: Delete) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;

    let default_branch = git.get_default_branch("origin")?;
    let branch_name = params.branch_name.as_str();

    if branch_name == default_branch || branch_name.ends_with(&format!("/{}", default_branch)) {
        return Err(format!("Cannot delete '{}'!", branch_name).into());
    }

    git.full_delete_branch(branch_name)?;

    println!("Branch {} deleted.", branch_name);

    Ok(())
}
