use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Params;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;
    let all_files = git.get_unstaged_files()?;
    let files: Vec<_> = all_files
        .iter()
        .filter(|x| !x.ends_with("Cargo.lock"))
        .collect();

    Err(Command::new("git")
        .arg("checkout")
        .args(params.args)
        .arg("--")
        .args(files)
        .exec()
        .into())
}
