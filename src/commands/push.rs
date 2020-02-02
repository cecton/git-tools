use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Params;

pub fn run(params: Params) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;

    Err(match (git.branch_name.as_ref(), git.upstream.as_ref()) {
        (Some(name), None) => Command::new("git")
            .arg("push")
            .args(&["--set-upstream", "origin", name])
            .args(params.args)
            .exec()
            .into(),
        _ => Command::new("git").arg("push").args(params.args).exec().into(),
    })
}
