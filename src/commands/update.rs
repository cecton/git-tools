use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Update;

pub fn run(params: Update) -> Result<(), Box<dyn std::error::Error>> {
    let git = Git::open()?;

    if params.deps {
        update_dependencies(git)?
    } else {
        update_branch(git, params)?
    }

    Ok(())
}

fn update_dependencies(mut git: Git) -> Result<(), Box<dyn std::error::Error>> {
    let cargo_update = Command::new("cargo").arg("update").status()?;

    if !cargo_update.success() {
        return Err("Command `cargo update` failed!".into());
    }

    git.commit_files("Update Cargo.lock", &["Cargo.lock"])?;

    Ok(())
}

fn update_branch(mut git: Git, params: Update) -> Result<(), Box<dyn std::error::Error>> {
    let default_branch = git.get_default_branch("origin")?;
    let top_commit = params.revision.clone().unwrap_or_else(|| default_branch);

    if git.has_file_changes()? {
        return Err("The repository has not committed changes, aborting.".into());
    }

    let mut rev_list = git.rev_list("HEAD", top_commit.as_str(), true)?;

    if rev_list.is_empty() {
        println!("Your branch is already up-to-date.");
        return Ok(());
    }

    let mut skipped = 0;
    let mut last_failing_revision: Option<String> = None;
    while let Some(revision) = rev_list.pop() {
        let message = format!("Merge commit {} (no conflict)\n\n", revision,);

        if let Some((_, cargo_lock_conflict)) =
            git.merge_no_conflict(revision.as_str(), message.as_str())?
        {
            println!(
                "All the commits to {} have been merged successfully without conflict",
                revision
            );
            if cargo_lock_conflict {
                println!("WARNING: conflict with Cargo.lock detected. Run `cargo git update --deps` to fix it.");
            }

            break;
        } else {
            skipped += 1;
            last_failing_revision = Some(revision.clone());
        }
    }

    if params.no_merge {
        return Ok(());
    } else if let Some(revision) = last_failing_revision {
        println!(
            "Your current branch is still behind '{}' by {} commit(s).",
            top_commit, skipped
        );
        println!("First merge conflict detected on: {}", revision);

        let message = format!("Merge commit {} (conflicts)\n\n", revision,);

        return Err(Command::new("git")
            .args(&[
                "merge",
                "--no-ff",
                revision.as_str(),
                "-m",
                message.as_str(),
            ])
            .args(params.merge_args)
            .exec()
            .into());
    } else {
        println!("Nothing more to merge. Your branch is up-to-date.");
    }

    Ok(())
}
