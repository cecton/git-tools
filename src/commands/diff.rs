use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Params;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Git::open()?;
    let all_files = repo.get_unstaged_files()?;
    let files: Vec<_> = all_files
        .iter()
        .filter(|x| !x.ends_with("Cargo.lock"))
        .collect();

    Err(Command::new("git")
        .arg("diff")
        .args(params)
        .arg("--")
        .args(files)
        .exec()
        .into())
}
