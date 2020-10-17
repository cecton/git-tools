Available Git subcommands
=========================

 *  [`git fork`](#git-fork)

    Create a new branch based on the default branch (usually `origin/main`).

 *  [`git push2`](#git-push2)

    Push a branch and set the upstream if not already set.

 *  [`git try-merge`](#git-try-merge)

    Does like a `git merge origin/main` but helps you resolve the conflicting
    commits one by one instead than having to solve them altogether like
    `git merge`.

git-try-merge
=============

Does like a `git merge origin/main` but helps you resolve the conflicting
commits one by one instead than having to solve them altogether like
`git merge`.

Synopsis
--------

```bash
git try-merge
# 1.  Merge as many non-conflicting commits as possible under one merge commit
#     (if any)
# 2.  Merge the first conflicting commit alone
#     (if any)
#
# Then you need to repeat the command `git try-merge` until your branch is
# fully updated.
#
# The point: all the conflicting commits will be merged one-by-one which will
# allow you to fully understand the reason of the conflict and solve them
# separately. (A bit like `git rebase` would do.)
```

There is no real equivalent purely with Git's CLI. This is the closest:

```bash
git fetch
git merge origin/main
# Then you will solve all the conflicts of all the commits in one commit,
# no matter how many commits are conflicting.
```

Installation
------------

```bash
cargo install git-tools --bin git-try-merge
```

git-fork
========

Create a new branch based on the default branch (usually `origin/main`).

Synopsis
--------

```bash
git fork new-branch

# This command will:
#  -  make sure there is no uncommitted changes (clean state)
#  -  fetch (update) origin/main (or your default branch)
#  -  create a new branch "new-branch" that will be based on origin/main
#  -  checkout on this new branch
```

More or less equivalent to:

```bash
git checkout main
git pull --ff-only
git checkout -b new-branch
```

### Implementation notes

The local branch main will not be updated. In fact, you don't even
need a local branch main. The exact equivalent with Git would be more
something like this:

```bash
git fetch origin main
# <ensure manually no uncommitted changes are pending>
git branch -f new-branch origin/main
git checkout new-branch
```

Installation
------------

```bash
cargo install git-tools --bin git-fork
```

git-push2
=========

Push a branch and set the upstream if not already set.

Synopsis
--------

```bash
git push2
```

This is the equivalent of:

```bash
git push
# if it fails:
git push --set-upstream origin new-branch
```

Installation
------------

```bash
cargo install git-tools --bin git-push2
```



Usage
-----

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

    Create a new branch based on <from-branch> (origin/main by default) and
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
    the parent branch is missing, origin/main is used.
