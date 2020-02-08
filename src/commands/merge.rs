use crate::git::Git;
use crate::Merge;

pub fn run(params: Merge) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let current_branch = if let Some(name) = git.branch_name.as_ref() {
        name.clone()
    } else {
        return Err("Cannot merge if you are not in a branch!".into());
    };

    let branch_name = params.branch_name.as_str();

    if let Some((_, cargo_lock_conflict)) = git.merge_no_conflict(
        branch_name,
        format!("Merge branch '{}' into {}", branch_name, current_branch).as_str(),
    )? {
        if cargo_lock_conflict {
            println!("WARNING: conflict with Cargo.lock detected. Run `cargo git update --deps` to fix it.");
        }
    } else {
        return Err("Merge conflict detected, aborted.".into());
    }

    if branch_name == "master" || branch_name.contains("/") {
        return Err(format!(
            "The branch '{}' has been merged but not deleted!",
            branch_name
        )
        .into());
    }

    git.full_delete_branch(branch_name)?;

    println!("The branch '{}' has been merged and deleted.", branch_name);

    Ok(())
}
