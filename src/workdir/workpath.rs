use std::{ops::Deref, borrow::Borrow, path::{Path, PathBuf}, fmt};

use crate::Error;

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WorkPath(str);

#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
pub struct WorkPathBuf(String);

impl WorkPath {
    unsafe fn from_str(slice: &str) -> &Self {
        std::mem::transmute(slice)
    }

    pub fn strip_prefix(&self, prefix: &WorkPath) -> Option<&Self> {
        if let Some(suffix) = self.0.strip_prefix(&prefix.0) {
            if suffix.is_empty() {
                unsafe { Some(Self::from_str(suffix)) }
            }
            else {
                unsafe { Some(Self::from_str(&suffix[1..])) }
            }
        }
        else {
            None
        }
    }

    pub fn strip_suffix(&self, suffix: &WorkPath) -> Option<&Self> {
        if let Some(suffix) = self.0.strip_suffix(&suffix.0) {
            if suffix.is_empty() {
                unsafe { Some(Self::from_str(suffix)) }
            }
            else {
                unsafe { Some(Self::from_str(&suffix[..suffix.len() - 1])) }
            }
        }
        else {
            None
        }
    }

    pub fn parent(&self) -> Option<&Self> {
        if self.0.is_empty() {
            None
        }
        else if let Some(last_sep_idx) = self.0.rfind('/') {
            let slice = &self.0[..last_sep_idx];
            unsafe { Some(Self::from_str(slice)) }
        }
        else {
            unsafe {
                Some(Self::from_str(""))
            }
        }
    }

    pub fn file_name(&self) -> &Self {
        if let Some(last_sep_idx) = self.0.rfind('/') {
            let slice = &self.0[last_sep_idx + 1..];
            unsafe { Self::from_str(slice) }
        }
        else {
            self
        }
    }

    pub fn partition(&self) -> (&Self, Option<&Self>) {
        if let Some((first, rest)) = self.0.split_once('/') {
            unsafe {
                (Self::from_str(first),
                    Some(Self::from_str(rest))
                )
            }
        }
        else {
            (self, None)
        }
    }
}

impl Borrow<str> for WorkPath {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl ToOwned for WorkPath {
    type Owned = WorkPathBuf;
    
    fn to_owned(&self) -> Self::Owned {
        Self::Owned::from(self)
    }
}

impl fmt::Display for WorkPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl WorkPathBuf {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_ref()
    }

    pub fn as_path(&self) -> &Path {
        self.0.as_ref()
    }
}

impl Deref for WorkPathBuf {
    type Target = WorkPath;

    fn deref(&self) -> &Self::Target {
        unsafe {
            WorkPath::from_str(self.0.as_str())
        }
    }
}

impl Borrow<WorkPath> for WorkPathBuf {
    fn borrow(&self) -> &WorkPath {
        self
    }
}

impl Borrow<String> for WorkPathBuf {
    fn borrow(&self) -> &String {
        &self.0
    }
}

impl AsRef<Path> for WorkPathBuf {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl From<&WorkPath> for WorkPathBuf {
    fn from(value: &WorkPath) -> Self {
        WorkPathBuf(value.0.to_owned())
    }
}

impl TryFrom<&str> for WorkPathBuf {
    type Error = Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = value.replace('\\', "/");

        if path.starts_with('/') || path.contains(':') {
            return Err(Error::PathIsAbsolute);
        }

        let normalized_path =
            path.split('/')
            .filter_map(|part| {
                if part.is_empty() {
                    None
                }
                else if [".", "..", ".git"].contains(&part) {
                    Some(Err(Error::ForbiddenPathComponent(part.to_owned())))
                }
                else {
                    Some(Ok(part))
                }
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("/");
        
        Ok(WorkPathBuf(normalized_path))
    }
}

impl TryFrom<String> for WorkPathBuf {
    type Error = Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<&Path> for WorkPathBuf {
    type Error = Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if let Some(path) = value.to_str() {
            Self::try_from(path)
        }
        else {
            Err(Error::InvalidPath)
        }
    }
}

impl TryFrom<PathBuf> for WorkPathBuf {
    type Error = Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(value.as_path())
    }
}

impl TryFrom<&[u8]> for WorkPathBuf {
    type Error = Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let path = std::str::from_utf8(value)?;
        Self::try_from(path)
    }
}

impl fmt::Display for WorkPathBuf {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
