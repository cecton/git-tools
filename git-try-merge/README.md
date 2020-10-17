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
cargo install git-try-merge
```
