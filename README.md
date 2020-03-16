cargo-git
=========

An opinionated helper command to use git with cargo. This does not replace the
git command but should be used in conjunction with.

This program is in testing, please use with care! The author will not be
responsible if you lose any data! This program is distributed in the hope that
it will be useful, but WITHOUT ANY WARRANTY.

Usage
-----

1.  Starting a new branch.

    Usually you want to fetch first and then create a branch based on master.
    To do this with Git you will do:

    ```
    git checkout master
    git pull --ff-only
    git checkout -b new-branch
    ```

    With cargo-git, from any branch you can directly do:

    ```
    # alias cg="cargo git"
    cg fork new-branch
    ```

    This command will:
     -  make sure there is no uncommitted changes (clean state)
     -  fetch (update) master from the remote
     -  create a new branch "new-branch" that will be based on origin/master

    Note: the local branch master will not be updated. In fact, you don't even
    need a local branch master. The exact equivalent with Git would be more
    something like this:

    ```
    git fetch
    # <ensure manually no uncommitted changes are pending>
    git branch -f new-branch origin/master
    git checkout new-branch
    ```

2.  Pushing the branch

    Usually you want to push with the same name remotely than locally. To do
    this with Git you will do:

    ```
    git push
    # if it fails:
    git push --set-upstream origin new-branch
    ```

    With cargo-git, only one command is necessary because it will automatically
    set the upstream if it wasn't set:

    ```
    # alias cg="cargo git"
    cg push
    ```

3.  Updating the branch

    Usually you want to update your local branch with origin/master. To do this
    with Git you will do:

    ```
    git fetch
    git merge origin/master
    # then you will solve all the conflicts of all the commits in one commit,
    # no matter how many commits are conflicting
    ```

    With cargo-git, you would do:

    ```
    # alias cg="cargo git"
    cg update
    # either there is no conflict and you're done
    #
    # or: all the non-conflicting commits will be merged in a single merge
    # commit until the first conflicting commit is reached THEN the first
    # conflicting commit will be merged alone, leaving you in a merge state
    #
    # this will allow you to solve the first conflict separetaly in its own
    # merge commit
    #
    # repeat the command `cg update` until there is nothing more to merge
    ```

    Note: all the conflicting commits will be merged one-by-one which will
    allow you to fully understand the reason of the conflict and solve them
    separately. (A bit like `git rebase` would do.)

4.  Deleting a branch

    Usually you want to delete the branch locally and remotely when you're
    done. In Git you would do:

    ```
    git branch -d new-branch
    git push origin :new-branch
    ```

    With cargo-git you can do both at once:

    ```
    # alias cg="cargo git"
    cg delete new-branch
    ```

List of the Available Commands
------------------------------

 *  `cargo git fork <new-branch> [<from-branch>]`

    Create a new branch based on <from-branch> (origin/master by default) and
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

*  `cargo git check [<revision>]`

    Check that HEAD can be merged without conflict

    If the `revision` argument is not provided, the parent branch is used. If
    the parent branch is missing, origin/master is used.
