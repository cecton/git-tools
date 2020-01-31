pub use git2::{Oid, Repository};
use git2::{Error, StatusOptions};
use regex::Regex;

pub struct Git {
    repo: Repository,
    head_message: String,
    head_hash: String,
}

impl Git {
    pub fn open() -> Result<Git, Error> {
        let repo = Repository::open(".")?;
        let head_message;
        let head_hash;

        {
            let object = repo.revparse_single("HEAD")?;
            let commit = object.as_commit().unwrap();
            head_message = commit.message().unwrap().to_string();
            head_hash = hash_from_oid(object.id());
        }

        Ok(Git {
            repo,
            head_message,
            head_hash,
        })
    }

    pub fn get_forked_hash(&self) -> Result<Option<String>, Error> {
        let re = Regex::new(r"^\s*Forked at:\s*(\w+)").unwrap();

        if let Some(captures) = &re.captures(self.head_message.as_str()) {
            Ok(Some(captures[0].to_string()))
        } else {
            Ok(None)
        }
    }

    pub fn get_parent_branch(&self) -> Result<Option<String>, Error> {
        let re = Regex::new(r"^\s*Parent branch:\s*(\w+)").unwrap();

        if let Some(captures) = &re.captures(self.head_message.as_str()) {
            Ok(Some(captures[0].to_string()))
        } else {
            Ok(None)
        }
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
}

pub fn hash_from_oid(oid: Oid) -> String {
    let slice = oid.as_bytes();
    let mut out = String::with_capacity(slice.len() * 2);

    for x in slice {
        out.push_str(format!("{:02x}", x).as_str());
    }

    out
}
