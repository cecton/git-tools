![Rust](https://github.com/cecton/git-tools/workflows/Rust/badge.svg)
[![Latest Version](https://img.shields.io/crates/v/git-tools.svg)](https://crates.io/crates/git-tools)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](http://opensource.org/licenses/MIT)
[![Docs.rs](https://docs.rs/git-tools/badge.svg)](https://docs.rs/git-tools)

##### Table of Contents

 *  [`git delete`](#git-delete)

    Delete a local branch and its upstream branch altogether.

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

git-delete
==========

Delete a local branch and its upstream branch altogether.

Synopsis
--------

```bash
git delete new-branch
```

This is the equivalent of:

```
git branch -d new-branch
git push origin :new-branch
```

Installation
------------

```bash
cargo install git-tools --bin git-delete
```
