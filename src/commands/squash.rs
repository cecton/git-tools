use crate::git::Git;
use crate::Squash;

pub fn run(params: Squash) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let revision = if let Some(rev) = params.revision {
        rev
    } else if let Some(rev) = git.get_forked_hash()? {
        rev
    } else {
        return Err("Could not find forked point and no revision provided!".into());
    };

    let hash = git.reset_soft(revision.as_str(), "moving for squashing")?;

    println!("HEAD moved to {}", hash);

    Ok(())
}
