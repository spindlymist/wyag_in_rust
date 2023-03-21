use anyhow::bail;

use crate::{
    Result,
    repo::Repository
};

/// Metadata about certain objects in a repository (namely, commits and annotated tags).
/// Includes the name and email of the author as well as a descriptive message.
pub struct ObjectMetadata {
    pub author_name: String,
    pub author_email: String,
    pub message: String,
}

impl ObjectMetadata {
    /// Constructs an `ObjectMetadata` object with the given message and the author info
    /// from `repo`'s config file. Fails if no user name or email is configured.
    pub fn new(repo: &Repository, message: String) -> Result<ObjectMetadata> {
        let author_name = match repo.get_config("user", "name") {
            Some(val) => val.to_owned(),
            None => bail!("No user name configured"),
        };

        let author_email = match repo.get_config("user", "email") {
            Some(val) => val.to_owned(),
            None => bail!("No user email configured"),
        };

        Ok(ObjectMetadata {
            author_name,
            author_email,
            message
        })
    }

    pub fn author_line(&self) -> String {
        format!("{} <{}>", self.author_name, self.author_email)
    }
}
