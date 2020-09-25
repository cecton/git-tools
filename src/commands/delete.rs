use crate::git::Git;
use crate::Delete;

pub fn run(params: Delete) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;

    let default_branch = git.get_default_branch("origin")?;
    let branch_name = params.branch_name.as_str();

    if branch_name == default_branch || branch_name.ends_with(&format!("/{}", default_branch)) {
        return Err(format!("Cannot delete '{}'!", branch_name).into());
    }

    git.full_delete_branch(branch_name)?;

    println!("Branch {} deleted.", branch_name);

    Ok(())
}
