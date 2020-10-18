#![allow(dead_code)]

use std::env::{current_dir, set_current_dir};
use std::path::{Path, PathBuf};

use git2::{
    Branch, BranchType, Commit, Config, Cred, CredentialType, Error, ErrorCode, FetchOptions,
    MergeOptions, PushOptions, RemoteCallbacks, Sort, StatusOptions,
};
pub use git2::{Oid, Repository};

pub struct Git {
    pub repo: Repository,
    pub head_message: String,
    pub head_hash: String,
    pub branch_name: Option<String>,
    pub upstream: Option<String>,
    pub config: Config,
}

impl Git {
    pub fn open() -> Result<Git, Error> {
        if let Some(path) = find_git_repository()? {
            set_current_dir(path).map_err(|e| Error::from_str(&e.to_string()))?;
        }

        let repo = Repository::open(".")?;
        let head_message;
        let head_hash;
        let branch_name;
        let upstream;

        {
            let (object, maybe_ref) = repo.revparse_ext("HEAD")?;
            let commit = object.as_commit().unwrap();
            head_message = commit.message().unwrap().to_string();
            head_hash = format!("{}", object.id());
            branch_name = maybe_ref.and_then(|x| {
                x.shorthand()
                    .filter(|&x| x != "HEAD")
                    .map(|x| x.to_string())
            });
            upstream = if let Some(name) = branch_name.as_ref() {
                if let Ok(remote_branch) = repo
                    .find_branch(name, BranchType::Local)
                    .and_then(|x| x.upstream())
                {
                    remote_branch.name()?.map(|x| x.to_string())
                } else {
                    None
                }
            } else {
                None
            };
        }

        let config = repo.config()?.snapshot()?;

        Ok(Git {
            repo,
            head_message,
            head_hash,
            branch_name,
            upstream,
            config,
        })
    }

    pub fn get_staged_and_unstaged_files(&self) -> Result<Vec<String>, Error> {
        let mut files = Vec::new();
        let mut options = StatusOptions::new();
        options.include_untracked(true);
        options.include_ignored(false);

        for entry in self.repo.statuses(Some(&mut options))?.iter() {
            files.push(entry.path().unwrap().to_string());
        }

        Ok(files)
    }

    pub fn branch(&self, name: &str, from: Option<&str>) -> Result<String, Error> {
        let object = self.repo.revparse_single(from.unwrap_or("HEAD"))?;
        let commit = object.as_commit().unwrap();
        let branch = self.repo.branch(name, &commit, false)?;

        Ok(branch.get().name().unwrap().to_string())
    }

    pub fn get_branch_hash(&self, branch_name: &str) -> Result<Option<String>, Error> {
        if let (_, Some(reference)) = self.repo.revparse_ext(branch_name)? {
            Ok(Some(format!("{}", reference.target().unwrap())))
        } else {
            Ok(None)
        }
    }

    pub fn get_default_branch(&self, remote: &str) -> Result<String, Error> {
        let reference = match self
            .repo
            .find_reference(format!("refs/remotes/{}/HEAD", remote).as_str())
        {
            Ok(x) => x,
            Err(err) if err.code() == ErrorCode::NotFound => {
                return Ok(format!("{}/master", remote))
            }
            Err(err) => return Err(err),
        };

        Ok(reference
            .symbolic_target()
            .expect("reference HEAD is not symbolic")
            .strip_prefix("refs/remotes/")
            .expect("invalid target")
            .to_string())
    }

    pub fn switch_branch(&mut self, branch_name: &str) -> Result<(), Error> {
        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let object = self.repo.revparse_single(branch_name)?;

        self.repo.checkout_tree(&object, None)?;
        self.repo.set_head(branch.get().name().unwrap())?;

        self.branch_name = Some(branch_name.to_string());
        self.head_hash = format!("{}", object.id());
        if let Ok(upstream) = branch.upstream() {
            self.upstream = upstream.name()?.map(|x| x.to_string());
        }

        Ok(())
    }

    pub fn commit_files(&mut self, message: &str, files: &[&str]) -> Result<Oid, Error> {
        let object = self.repo.revparse_single("HEAD")?;
        let commit = object.as_commit().unwrap();
        let old_tree = commit.tree()?;

        let mut treebuilder = self.repo.treebuilder(Some(&old_tree))?;
        for file in files {
            let oid = self.repo.blob_path(Path::new(file))?;
            treebuilder.insert(file, oid, 0o100644)?;
        }
        let tree_oid = treebuilder.write()?;
        let tree = self.repo.find_tree(tree_oid)?;

        let signature = self.repo.signature()?;
        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&commit],
        )?;

        let mut index = self.repo.index()?;
        index.update_all(files, None)?;
        self.repo.checkout_index(Some(&mut index), None)?;

        self.head_hash = format!("{}", oid);

        Ok(oid)
    }

    pub fn full_delete_branch(&self, branch_name: &str) -> Result<(), Error> {
        let mut local_branch = self.repo.find_branch(branch_name, BranchType::Local)?;

        if let Ok(remote_branch) = local_branch.upstream() {
            let (maybe_remote, name) = get_remote_and_branch(&remote_branch);
            let remote = maybe_remote.expect("remote branch");

            let mut remote_callbacks = RemoteCallbacks::new();
            let mut handler = CredentialHandler::new();
            remote_callbacks.credentials(move |x, y, z| handler.credentials_callback(x, y, z));

            let mut push_options = PushOptions::new();
            push_options.remote_callbacks(remote_callbacks);

            self.repo.find_remote(remote)?.push(
                &[&format!("+:refs/heads/{}", name)],
                Some(&mut push_options),
            )?;
        }

        local_branch.delete()?;

        Ok(())
    }

    pub fn has_file_changes(&self) -> Result<bool, Error> {
        let tree = self.repo.head()?.peel_to_tree()?;

        Ok(self
            .repo
            .diff_tree_to_workdir_with_index(Some(&tree), None)?
            .stats()?
            .files_changed()
            > 0)
    }

    pub fn check_no_conflict(&mut self, branch_name: &str) -> Result<Option<bool>, Error> {
        let mut cargo_lock_conflict = false;
        let our_object = self.repo.revparse_single("HEAD")?;
        let our = our_object.as_commit().expect("our is a commit");
        let their_object = self.repo.revparse_single(branch_name)?;
        let their = their_object.as_commit().expect("their is a commit");

        let mut options = MergeOptions::new();
        options.fail_on_conflict(false);

        let index = self.repo.merge_commits(&our, &their, Some(&options))?;
        let conflicts = index.conflicts()?.collect::<Result<Vec<_>, _>>()?;
        for conflict in conflicts {
            let their = conflict.their.expect("an index entry for their exist");
            let path = std::str::from_utf8(their.path.as_slice()).expect("valid UTF-8");

            if path == "Cargo.lock" {
                cargo_lock_conflict = true;
            } else {
                return Ok(None);
            }
        }

        Ok(Some(cargo_lock_conflict))
    }

    pub fn merge_no_conflict(
        &mut self,
        branch_name: &str,
        message: &str,
    ) -> Result<Option<(String, bool)>, Error> {
        let mut cargo_lock_conflict = false;
        let our_object = self.repo.revparse_single("HEAD")?;
        let our = our_object.as_commit().expect("our is a commit");
        let their_object = self.repo.revparse_single(branch_name)?;
        let their = their_object.as_commit().expect("their is a commit");

        let mut options = MergeOptions::new();
        options.fail_on_conflict(false);

        let mut index = self.repo.merge_commits(&our, &their, Some(&options))?;
        let conflicts = index.conflicts()?.collect::<Result<Vec<_>, _>>()?;
        for conflict in conflicts {
            let their = conflict.their.expect("an index entry for their exist");
            let path = std::str::from_utf8(their.path.as_slice()).expect("valid UTF-8");

            if path == "Cargo.lock" {
                use bitvec::prelude::*;

                let mut flags = BitVec::<Msb0, _>::from_element(their.flags);
                // NOTE: Reset stage flags
                // https://github.com/git/git/blob/master/Documentation/technical/index-format.txt
                flags[2..=3].set_all(false);
                let their = git2::IndexEntry {
                    flags: flags.as_slice()[0],
                    ..their
                };
                index.remove_path(Path::new("Cargo.lock")).unwrap();
                index.add(&their)?;
                cargo_lock_conflict = true;
            } else {
                return Ok(None);
            }
        }

        let oid = index.write_tree_to(&self.repo)?;
        let tree = self.repo.find_tree(oid)?;

        let signature = self.repo.signature()?;
        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&our, &their],
        )?;

        let mut checkout_builder = git2::build::CheckoutBuilder::new();
        checkout_builder.force();
        self.repo.checkout_head(Some(&mut checkout_builder))?;

        self.head_hash = format!("{}", oid);

        Ok(Some((self.head_hash.clone(), cargo_lock_conflict)))
    }

    pub fn rev_list(&self, from: &str, to: &str, reversed: bool) -> Result<Vec<String>, Error> {
        let mut revwalk = self.repo.revwalk()?;
        if reversed {
            revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE);
        } else {
            revwalk.set_sorting(Sort::TOPOLOGICAL);
        }

        let from_object = self.repo.revparse_single(from)?;
        let to_object = self.repo.revparse_single(to)?;
        revwalk.hide(from_object.id())?;
        revwalk.push(to_object.id())?;

        revwalk
            .map(|x| x.map(|x| format!("{}", x)))
            .collect::<Result<Vec<_>, Error>>()
    }

    pub fn update_upstream(&self, branch_name: &str) -> Result<(), Error> {
        let branch = self.repo.find_branch(branch_name, BranchType::Remote)?;
        let (maybe_remote_name, branch_name) = get_remote_and_branch(&branch);

        // TODO: this method fails if branch_name is not a remote branch
        //       this `if` statement makes no sense
        if let Some(remote_name) = maybe_remote_name {
            let mut remote_callbacks = RemoteCallbacks::new();
            let mut handler = CredentialHandler::new();
            remote_callbacks.credentials(move |x, y, z| handler.credentials_callback(x, y, z));

            let mut fetch_options = FetchOptions::new();
            fetch_options.remote_callbacks(remote_callbacks);

            self.repo.find_remote(remote_name)?.fetch(
                &[branch_name],
                Some(&mut fetch_options),
                None,
            )?;
        }

        Ok(())
    }

    pub fn ancestors(&self, rev: &str) -> Result<Ancestors, Error> {
        let object = self.repo.revparse_single(rev)?;
        let commit = object.peel_to_commit()?;

        Ok(Ancestors {
            current: Some(commit),
        })
    }

    pub fn squash(
        &mut self,
        parent_0: &str,
        parent_1: &str,
        message: &str,
    ) -> Result<String, Error> {
        let parent_0 = self.repo.revparse_single(parent_0)?.peel_to_commit()?;
        let parent_1 = self.repo.revparse_single(parent_1)?.peel_to_commit()?;
        let head = self.repo.revparse_single("HEAD")?.peel_to_commit()?;
        let tree = self.repo.find_tree(head.tree_id())?;

        // git reset --soft to the parent "0" commit
        if let (_, Some(mut reference)) = self
            .repo
            .revparse_ext(self.branch_name.as_ref().unwrap_or(&self.head_hash))?
        {
            reference.set_target(parent_0.id(), message)?;
        } else {
            self.repo.set_head_detached(parent_0.id())?;
        }

        // Make a commit with the current tree
        let signature = self.repo.signature()?;
        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&parent_0, &parent_1],
        )?;

        self.head_hash = format!("{}", oid);

        Ok(self.head_hash.clone())
    }
}

fn find_git_repository() -> Result<Option<PathBuf>, Error> {
    let mut path = current_dir().map_err(|e| Error::from_str(&e.to_string()))?;

    loop {
        if path.join(".git").exists() {
            return Ok(Some(path));
        }
        if !path.pop() {
            break;
        }
    }

    Ok(None)
}

fn get_remote_and_branch<'a>(branch: &'a Branch) -> (Option<&'a str>, &'a str) {
    let mut parts = branch
        .get()
        .shorthand()
        .expect("valid UTF-8")
        .rsplitn(2, '/');
    let branch_name = parts.next().unwrap();
    let maybe_remote_name = parts.next();

    (maybe_remote_name, branch_name)
}

struct CredentialHandler {
    second_handler: git2_credentials::CredentialHandler,
    first_attempt_failed: bool,
}

impl CredentialHandler {
    fn new() -> CredentialHandler {
        let git_config = git2::Config::open_default().unwrap();
        let second_handler = git2_credentials::CredentialHandler::new(git_config);

        CredentialHandler {
            second_handler,
            first_attempt_failed: false,
        }
    }

    fn credentials_callback(
        &mut self,
        url: &str,
        username_from_url: Option<&str>,
        allowed_types: CredentialType,
    ) -> Result<Cred, Error> {
        if !self.first_attempt_failed && allowed_types.contains(CredentialType::SSH_KEY) {
            self.first_attempt_failed = true;
            let user = users::get_current_username().expect("could not get username");
            let home_dir = dirs::home_dir().expect("could not get home directory");

            Cred::ssh_key(
                username_from_url.unwrap_or_else(|| user.to_str().unwrap()),
                Some(&home_dir.join(".ssh/id_rsa.pub")),
                &home_dir.join(".ssh/id_rsa"),
                None,
            )
        } else {
            self.second_handler
                .try_next_credential(url, username_from_url, allowed_types)
        }
    }
}

pub struct Ancestors<'a> {
    current: Option<Commit<'a>>,
}

impl<'a> Iterator for Ancestors<'a> {
    type Item = Commit<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.current.take().map(|this| {
            self.current = this.parent(0).ok();
            this
        })
    }
}
