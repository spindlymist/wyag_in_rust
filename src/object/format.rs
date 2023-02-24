use std::fmt;

use crate::error::Error;

#[derive(PartialEq, Eq, Debug)]
pub enum ObjectFormat {
    Blob,
    Commit,
    Tag,
    Tree,
}

impl fmt::Display for ObjectFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use ObjectFormat::*;

        let format_name = match self {
            Blob => "blob",
            Commit => "commit",
            Tag => "tag",
            Tree => "tree",
        };

        write!(f, "{format_name}")
    }
}

impl TryFrom<&str> for ObjectFormat {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        use ObjectFormat::*;

        match value {
            "blob" => Ok(Blob),
            "commit" => Ok(Commit),
            "tag" => Ok(Tag),
            "tree" => Ok(Tree),
            _ => Err(Error::UnrecognizedObjectFormat),
        }
    }
}
