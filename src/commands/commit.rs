use std::process::Command;
use std::os::unix::process::CommandExt;

use crate::Params;
use crate::git::Git;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Git::open()?;
    let mut message = "WIP\n\n".to_string();

    if let Some(hash) = repo.get_forked_hash()? {
        message.push_str(&format!("Forked at: {}", hash));
    }

    if let Some(branch) = repo.get_parent_branch()? {
        message.push_str(&format!("Parent branch: {}", branch));
    }

    Err(Command::new("git")
        .args(&["commit", "-m", message.as_str()])
        .args(params)
        .exec()
        .into())
}
