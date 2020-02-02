use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Params;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;
    let mut message = "WIP\n\n".to_string();

    // TODO: should probably look at passed commits too? Problem with cherry-picking

    if let Some(hash) = git.get_forked_hash()? {
        message.push_str(&format!("Forked at: {}\n", hash));
    }

    if let Some(branch) = git.get_parent_branch()? {
        message.push_str(&format!("Parent branch: {}\n", branch));
    }

    Err(Command::new("git")
        .args(&["commit", "-m", message.as_str()])
        .args(params.args)
        .exec()
        .into())
}
