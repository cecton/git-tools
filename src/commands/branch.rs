use crate::git::Git;
use crate::Branch;

pub fn run(params: Branch) -> Result<(), Box<dyn std::error::Error>> {
    let mut repo = Git::open()?;

    if params.delete {
        repo.full_delete_branch(params.branch_name.as_str())?;

        println!("Branch {} deleted.", params.branch_name);

        Ok(())
    } else {
        let mut message = "Initial commit\n\n".to_string();

        message.push_str(&format!("Forked at: {}\n", repo.head_hash));

        if let Some(name) = repo.branch_name.as_ref() {
            message.push_str(&format!("Parent branch: {}\n", name));
        }

        repo.branch(params.branch_name.as_str())?;
        repo.set_head(params.branch_name.as_str())?;
        repo.commit(message.as_str())?;

        println!("Branch {} created.", params.branch_name);

        Ok(())
    }
}
