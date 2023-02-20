use crate::{
    Error,
    Result,
    repo::Repository
};

pub struct ObjectMetadata {
    pub author_name: String,
    pub author_email: String,
    pub message: String,
}

impl ObjectMetadata {
    pub fn new(repo: &Repository, message: String) -> Result<ObjectMetadata> {
        let author_name = match repo.get_config("user", "name") {
            Some(val) => val.to_owned(),
            None => return Err(Error::MissingConfig("No user name configured".to_owned())),
        };

        let author_email = match repo.get_config("user", "email") {
            Some(val) => val.to_owned(),
            None => return Err(Error::MissingConfig("No user email configured".to_owned())),
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
