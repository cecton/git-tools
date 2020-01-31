cargo-git
=========

An opinionated helper command to use git with cargo. This does not replace the
git command but should be used in conjunction with.

 *  `cargo git branch <new_branch>`

    Create a new branch based on the current branch to switch to it

 *  `cargo git branch -d <existing_branch>`

    Deletes existing branch locally and remotely

 *  `cargo git merge <branch>`

    Merge branch to the current branch with a merge commit only
    (traditional merge). And delete the local and remote branch afterwards.

    This command will fail in case of merge conflict.

 *  `cargo git update`

    Update the current branch by merging the base branch to the current branch.

    This command merge the missing commits from the base branch to the current
    branch and stops right before it encounters a conflict.

    If the first commit to merge is conflicting, it does a merge alone of this
    commit, allowing the user to resolve it and commit.

    The command can be (is intended to be) repeated until the current branch
    has no missing commit.

    It ignores Cargo.lock conflicts by taking the Cargo.lock of the current
    branch.

 *  `cargo git update --rebase`

    Update the current branch by rebasing the current branch to the base
    branch.

    Classic rebase. When it ends, the branch is up-to-date, no need to run
    cargo git update again.

    It ignores Cargo.lock conflicts by taking the Cargo.lock of the current
    branch.

 *  `cargo git update --deps`

    Runs cargo update and commit only Cargo.lock alone

 *  `cargo git push`

    Try to push the current branch to its remote branch. If the remote branch
    does not exist, create one with the same name and set it on the local
    branch.

 *  `cargo git add [params]`

    Same as `git add` but doesn't go through Cargo.lock

 *  `cargo git diff [params]`

    Same as `git diff` but always ignore Cargo.lock

 *  `cargo git commit [params]`

    Commit the files added with the message WIP and add the parent branch in
    description.

 *  `cargo git checkout [params]`

    Exactly the same as `git checkout` but always ignore Cargo.lock

    `git checkout [params] -- (git diff --name-only | grep -v Cargo.lock)`

 *  `cargo git checkout --cargo [params]`

    `git checkout [params] -- (git diff --name-only | grep Cargo.toml)`

 *  `cargo git squash [<other-commit>]`

    `git reset --soft <other-commit|forking-commit>`

    This command allows squashing the current branch by reseting to the parent
    commit. This command should be followed ultimately by `git commit`.
