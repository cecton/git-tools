use std::process::Command;
use std::os::unix::process::CommandExt;

use crate::Params;
use crate::git::Git;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Git::open()?;
    let all_files = repo.get_unstaged_files()?;
    let files: Vec<_> = all_files.iter().filter(|x| !x.ends_with("Cargo.lock")).collect();

    Err(Command::new("git")
        .arg("checkout")
        .args(params)
        .arg("--")
        .args(files)
        .exec()
        .into())
}
