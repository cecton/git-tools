use crate::git::Git;
use crate::Squash;

pub fn run(params: Squash) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let revision = if let Some(rev) = params.revision {
        rev
    } else if let Some(parent) = git.get_parent_branch()? {
        let (_ahead, behind) = git.graph_ahead_behind("HEAD", parent.as_str())?;

        if behind > 0 {
            return Err(format!("The current branch is not up-to-date with {}.", parent).into());
        }

        parent
    } else {
        return Err("Could not find forked point and no revision provided!".into());
    };

    let hash = git.reset_soft(revision.as_str(), "moving for squashing")?;

    println!("HEAD moved to {}", hash);

    Ok(())
}
