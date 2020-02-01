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

    git.merge_no_conflict(
        branch_name,
        format!("Merge branch '{}' into {}", branch_name, current_branch).as_str(),
    )?;
    git.full_delete_branch(branch_name)?;

    println!("The branch '{}' has been merged and deleted.", branch_name);

    Ok(())
}
