use crate::git::Git;
use crate::Fork;

pub fn run(params: Fork) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    if git.has_file_changes()? {
        return Err("The repository has not committed changes, aborting.".into());
    }

    let branch_name = params.branch_name.as_str();
    let default_branch = git.get_default_branch("origin")?;
    let name = params
        .from
        .as_deref()
        .unwrap_or_else(|| default_branch.as_str());

    let mut message = "Initial commit\n\n".to_string();

    if name.contains('/') {
        git.update_upstream(name)?;
    }

    match git.get_branch_hash(name)? {
        // name is really a branch
        Some(hash) => {
            message.push_str(&format!("Forked at: {}\n", hash));
            message.push_str(&format!("Parent branch: {}\n", name));

            git.branch(branch_name, Some(hash.as_str()))?;
        }
        // name was not a branch
        None => {
            message.push_str(&format!("Forked at: {}\n", name));
            message.push_str("No parent branch.\n");

            git.branch(branch_name, Some(name))?;
        }
    }

    git.switch_branch(branch_name)?;
    git.commit(message.as_str())?;

    println!("Branch {} created.", branch_name);

    Ok(())
}
