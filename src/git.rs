pub use git2::{Oid, Repository};
use git2::Error;
use regex::Regex;

pub struct Git(Repository);

impl Git {
    pub fn open() -> Result<Git, Error> {
        Ok(Git(Repository::open(".")?))
    }

    pub fn get_forked_hash(&self) -> Result<Option<String>, Error> {
        let object = self.0.revparse_single("HEAD")?;
        let commit = object.as_commit().unwrap();
        let message = commit.message().unwrap();
        let re = Regex::new(r"^\s*Forked at:\s*(\w+)").unwrap();

        if let Some(captures) = &re.captures(message) {
            Ok(Some(captures[0].to_string()))
        } else {
            Ok(None)
        }
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
