use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Update;

pub fn run(params: Update) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let (forked_at, parent_branch) = git.get_parent()?;
    let top_commit = params.revision.clone().or_else(|| parent_branch.clone());

    if params.deps {
        let cargo_update = Command::new("cargo").arg("update").status()?;

        if !cargo_update.success() {
            return Err("Command `cargo update` failed!".into());
        }

        git.commit_files("Update Cargo.lock", &["Cargo.lock"])?;
    } else if let Some(top_commit) = top_commit {
        if git.has_file_changes()? {
            return Err("The repository has not committed changes, aborting.".into());
        }

        let parent_branch = if let Some(parent) = parent_branch.as_ref() {
            if parent.contains('/') && params.revision.is_none() {
                git.update_upstream(parent)?;
            }

            format!("Parent branch: {}\n", parent)
        } else {
            String::new()
        };

        let mut rev_list = git.rev_list("HEAD", top_commit.as_str(), true)?;

        if rev_list.is_empty() {
            println!("Your branch is already up-to-date.");
            return Ok(());
        }

        let forked_at = if let Some(hash) = forked_at {
            format!("Forked at: {}\n", hash)
        } else {
            String::new()
        };

        let mut skipped = 0;
        let mut last_failing_revision: Option<String> = None;
        while let Some(revision) = rev_list.pop() {
            let mut message = format!("Merge commit {} (no conflict)\n\n", revision,);
            message.push_str(parent_branch.as_str());
            message.push_str(forked_at.as_str());

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

            let mut message = format!("Merge commit {} (conflicts)\n\n", revision,);
            message.push_str(parent_branch.as_str());
            message.push_str(forked_at.as_str());

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
    } else {
        return Err("Could not find parent branch and no revision specified!".into());
    }

    Ok(())
}
