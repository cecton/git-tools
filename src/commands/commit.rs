use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Params;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;
    let mut message = "WIP\n\n".to_string();

    let (forked_at, parent_branch) = git.get_parent()?;

    if let Some(hash) = forked_at {
        message.push_str(&format!("Forked at: {}\n", hash));
    }

    if let Some(branch) = parent_branch {
        message.push_str(&format!("Parent branch: {}\n", branch));
    }

    Err(Command::new("git")
        .args(&["commit", "-m", message.as_str()])
        .args(params.args)
        .exec()
        .into())
}
