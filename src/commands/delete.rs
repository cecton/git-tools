use crate::git::Git;
use crate::Delete;

pub fn run(params: Delete) -> Result<(), Box<dyn std::error::Error>> {
    let repo = Git::open()?;

    repo.full_delete_branch(params.branch_name.as_str())?;

    println!("Branch {} deleted.", params.branch_name);

    Ok(())
}
