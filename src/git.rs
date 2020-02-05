use std::env::{current_dir, set_current_dir};
use std::path::{Path, PathBuf};

use git2::{
    BranchType, Cred, CredentialType, Error, FetchOptions, MergeOptions, RemoteCallbacks, Sort,
    StatusOptions,
};
pub use git2::{Oid, Repository};
use regex::Regex;

pub struct Git {
    pub repo: Repository,
    pub head_message: String,
    pub head_hash: String,
    pub branch_name: Option<String>,
    pub upstream: Option<String>,
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
            head_hash = hash_from_oid(object.id());
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

        Ok(Git {
            repo,
            head_message,
            head_hash,
            branch_name,
            upstream,
        })
    }

    pub fn get_parent(&self) -> Result<(Option<String>, Option<String>), Error> {
        let re_forked_at = Regex::new(r"(?m)^\s*Forked at:\s*(\S+)").unwrap();
        let re_parent_branch = Regex::new(r"(?m)^\s*Parent branch:\s*(\S+)").unwrap();

        let mut revwalk = self.repo.revwalk()?;
        revwalk.set_sorting(Sort::TOPOLOGICAL);
        revwalk.push_head()?;

        let mut forked_at = None;
        let mut parent_branch = None;
        let mut count = 0;
        for oid in revwalk {
            let oid = oid?;
            let commit = self.repo.find_commit(oid)?;
            let message = commit.message().unwrap();

            if forked_at.is_none() {
                forked_at = re_forked_at
                    .captures(message)
                    .as_ref()
                    .map(|captures| captures[1].to_string());
            }
            if parent_branch.is_none() {
                parent_branch = re_parent_branch
                    .captures(message)
                    .as_ref()
                    .map(|captures| captures[1].to_string());
            }

            if forked_at.is_some() && parent_branch.is_some() {
                break;
            }

            count += 1;
            if count >= 30 {
                break;
            }
        }

        Ok((forked_at, parent_branch))
    }

    pub fn get_unstaged_files(&self) -> Result<Vec<String>, Error> {
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
            Ok(Some(hash_from_oid(reference.target().unwrap())))
        } else {
            Ok(None)
        }
    }

    pub fn switch_branch(&mut self, branch_name: &str) -> Result<(), Error> {
        let branch = self.repo.find_branch(branch_name, BranchType::Local)?;
        let object = self.repo.revparse_single(branch_name)?;

        self.repo.checkout_tree(&object, None)?;
        self.repo.set_head(branch.get().name().unwrap())?;

        self.branch_name = Some(branch_name.to_string());
        self.head_hash = hash_from_oid(object.id());
        if let Ok(upstream) = branch.upstream() {
            self.upstream = upstream.name()?.map(|x| x.to_string());
        }

        Ok(())
    }

    pub fn reset_soft(&mut self, revision: &str, message: &str) -> Result<String, Error> {
        let object = self.repo.revparse_single(revision)?;
        let oid = object.id();

        if let (_, Some(mut reference)) = self
            .repo
            .revparse_ext(self.branch_name.as_ref().unwrap_or(&self.head_hash))?
        {
            reference.set_target(oid, message)?;
        } else {
            self.repo.set_head_detached(oid)?;
        }

        self.head_hash = hash_from_oid(oid);

        Ok(self.head_hash.clone())
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

        self.head_hash = hash_from_oid(oid);

        Ok(oid)
    }

    pub fn commit(&mut self, message: &str) -> Result<Oid, Error> {
        let signature = self.repo.signature()?;
        let object = self.repo.revparse_single("HEAD")?;
        let commit = object.as_commit().unwrap();
        let tree = commit.tree()?;

        let oid = self.repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            message,
            &tree,
            &[&commit],
        )?;

        self.head_hash = hash_from_oid(oid);

        Ok(oid)
    }

    pub fn full_delete_branch(&self, branch_name: &str) -> Result<(), Error> {
        let mut local_branch = self.repo.find_branch(branch_name, BranchType::Local)?;

        if let Ok(mut remote_branch) = local_branch.upstream() {
            remote_branch.delete()?;
        }

        local_branch.delete()?;

        Ok(())
    }

    pub fn graph_ahead_behind(&self, from: &str, to: &str) -> Result<(usize, usize), Error> {
        let from = self
            .repo
            .revparse_single(from)?
            .into_commit()
            .expect("from is not a commit");
        let to = self
            .repo
            .revparse_single(to)?
            .into_commit()
            .expect("from is not a commit");

        self.repo.graph_ahead_behind(from.id(), to.id())
    }

    pub fn merge_no_conflict(&mut self, branch_name: &str, message: &str) -> Result<String, Error> {
        let our_object = self.repo.revparse_single("HEAD")?;
        let our = our_object.as_commit().unwrap();
        let their_object = self.repo.revparse_single(branch_name)?;
        let their = their_object.as_commit().unwrap();
        let a_commit = self.repo.find_annotated_commit(their_object.id())?;

        let mut options = MergeOptions::new();
        options.fail_on_conflict(true);
        self.repo.merge(&[&a_commit], Some(&mut options), None)?;
        let mut index = self.repo.index()?;
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
        self.repo.cleanup_state()?;

        self.head_hash = hash_from_oid(oid);

        Ok(self.head_hash.clone())
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
            .map(|x| x.map(|y| hash_from_oid(y)))
            .collect::<Result<Vec<_>, Error>>()
    }

    pub fn update_upstream(&self, branch: &str) -> Result<(), Error> {
        let mut parts = branch.rsplitn(2, '/');
        let branch_name = parts.next().unwrap();
        let maybe_remote_name = parts.next();

        if let Some(remote_name) = maybe_remote_name {
            let mut remote_callbacks = RemoteCallbacks::new();
            remote_callbacks.credentials(credentials_callback);

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
}

pub fn hash_from_oid(oid: Oid) -> String {
    let slice = oid.as_bytes();
    let mut out = String::with_capacity(slice.len() * 2);

    for x in slice {
        out.push_str(format!("{:02x}", x).as_str());
    }

    out
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

fn credentials_callback(
    _url: &str,
    username_from_url: Option<&str>,
    allowed_types: CredentialType,
) -> Result<Cred, Error> {
    if allowed_types.contains(CredentialType::SSH_KEY) {
        let user = users::get_current_username().expect("could not get username");
        let home_dir = dirs::home_dir().expect("could not get home directory");

        Cred::ssh_key(
            username_from_url.unwrap_or(user.to_str().unwrap()),
            Some(&home_dir.join(".ssh/id_rsa.pub")),
            &home_dir.join(".ssh/id_rsa"),
            None,
        )
    } else {
        todo!();
    }
}
