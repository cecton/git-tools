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
