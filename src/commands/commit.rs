use std::process::Command;
use std::os::unix::process::CommandExt;

use crate::Params;
use crate::git::Git;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Git::open()?;

    let message = if let Some(hash) = repo.get_forked_hash()? {
        format!("WIP\n\nForked at: {}", hash)
    } else {
        "WIP".to_string()
    };

    Err(Command::new("git")
        .args(&["commit", "-m", message.as_str()])
        .args(params)
        .exec()
        .into())
}
