cargo-git
=========

An opinionated helper command to use git with cargo. This does not replace the
git command but should be used in conjunction with.

This program is in testing, please use with care! The author will not be
responsible if you lose any data! This program is distributed in the hope that
it will be useful, but WITHOUT ANY WARRANTY.

 *  `cargo git fork <new-branch> [<from-branch>]`

    Create a new branch based on <from-branch> (current branch by default) and
    switch to it. Also make an init commit to track the forking branch and
    commit it came from.

 *  `cargo git delete <existing-branch>`

    Deletes existing branch locally and its upstream.

 *  `cargo git merge <branch>`

    Merge branch to the current branch with a merge commit only
    (traditional merge). And delete the local and remote branch afterwards.

    This command will fail if there is any conflict. (If the branch given is
    not up-to-date enough with the current branch.

 *  `cargo git update`

    Update the current branch by merging the parent branch to the current
    branch.

    This command merge the missing commits from the base branch to the current
    branch and stops right before it encounters a conflict.

    If the first commit to merge is conflicting, it does a merge alone of this
    commit, allowing the user to resolve it and commit.

    The command can be (is intended to be) repeated until the current branch
    has no missing commit.

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

 *  `cargo git squash [<other-commit>]`

    `git reset --soft <other-commit|forking-branch>`

    This command allows squashing the current branch by reseting to the parent
    branch. This command should be followed ultimately by `git commit`.

    This command will fail if the current branch is not up-to-date with the
    parent.
