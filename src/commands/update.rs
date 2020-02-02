use std::os::unix::process::CommandExt;
use std::process::Command;

use crate::git::Git;
use crate::Update;

pub fn run(params: Update) -> Result<(), Box<dyn std::error::Error>> {
    let mut git = Git::open()?;

    let current = git.branch_name.as_ref().unwrap_or(&git.head_hash).clone();

    if params.deps {
        let cargo_update = Command::new("cargo").arg("update").status()?;

        if !cargo_update.success() {
            return Err("Command `cargo update` failed!".into());
        }

        git.commit_files("Update Cargo.lock", &["Cargo.lock"])?;
    } else if let Some(parent) = git.get_parent_branch()?.as_ref() {
        let mut rev_list = git.rev_list("HEAD", parent, true)?;

        if rev_list.is_empty() {
            println!("Your branch is already up-to-date.");
            return Ok(());
        }

        let forked_at = if let Some(hash) = git.get_forked_hash()? {
            format!("Forked at: {}\n", hash)
        } else {
            String::new()
        };

        let mut skipped = 0;
        let mut last_failing_revision: Option<String> = None;
        while let Some(revision) = rev_list.pop() {
            let mut message = format!(
                "Update branch '{}' from parent '{}'\n\nCommit: {}\nParent branch: {}\n",
                current, parent, revision, parent,
            );
            message.push_str(forked_at.as_str());

            if git
                .merge_no_conflict(revision.as_str(), message.as_str())
                .is_ok()
            {
                println!("Succeeded to merge: {}", revision);

                if skipped == 0 {
                    println!("Nothing more to merge. Your branch is up-to-date.");

                    return Ok(());
                } else {
                    let revision = last_failing_revision.unwrap();

                    println!(
                        "Your current branch is still behind '{}' by {} commit(s).",
                        parent, skipped
                    );

                    let mut message = format!(
                        "Update branch '{}' from parent '{}'\n\nCommit: {}\nParent branch: {}\n",
                        current, parent, revision, parent,
                    );
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
                }
            } else {
                println!("Merge conflict detected on: {}", revision);
                skipped += 1;
                last_failing_revision = Some(revision.clone());
            }
        }

        return Err("All revisions have conflicts!".into());
    } else {
        return Err("Could not find parent branch!".into());
    }

    Ok(())
}
