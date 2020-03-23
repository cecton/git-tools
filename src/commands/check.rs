use crate::git::Git;
use crate::Check;

pub fn run(params: Check) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let (_forked_at, parent_branch) = git.get_parent()?;

    let parent = params
        .revision
        .as_ref()
        .or_else(|| parent_branch.as_ref())
        .map(|x| x.as_str())
        .unwrap_or("origin/master");

    if git.has_file_changes()? {
        return Err("The repository has not committed changes, aborting.".into());
    }

    if parent.contains('/') {
        git.update_upstream(parent)?;
    }

    let mut rev_list = git.rev_list("HEAD", parent, true)?;

    if rev_list.is_empty() {
        println!("Your branch is already up-to-date.");
        return Ok(());
    }

    let mut last_failing_revision: Option<String> = None;
    while let Some(revision) = rev_list.pop() {
        if let Some(cargo_lock_conflict) = git.check_no_conflict(revision.as_str())? {
            if cargo_lock_conflict {
                println!("WARNING: conflict with Cargo.lock detected. Run `cargo git update --deps` to fix it.");
            }

            break;
        } else {
            last_failing_revision = Some(revision.clone());
        }
    }

    if let Some(revision) = last_failing_revision {
        Err(format!("Conflict detected on {}", revision).into())
    } else {
        println!("No conflict detected.");

        Ok(())
    }
}
