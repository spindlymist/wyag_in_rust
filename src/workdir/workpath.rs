use std::{ops::Deref, borrow::Borrow, path::{Path, PathBuf}, fmt, ffi::OsString};

use super::WorkDirError;

/// A normalized path relative to a working directory.
/// 
/// Viewed as a string, a `WorkPath` is always valid Utf-8, always uses `/` as a path separator,
/// never begins or ends with a slash, and never contains the components `.git`, `.`, or `..`.
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct WorkPath(str);

/// The owned variant of a [`WorkPath`].
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Debug)]
pub struct WorkPathBuf(String);

impl WorkPath {
    /// Turns a `str` slice into a `WorkPath`. This works because, thanks to `repr(transparent)`,
    /// the two types are guaranteed identical in memory.
    unsafe fn from_str(slice: &str) -> &Self {
        std::mem::transmute(slice)
    }

    /// Returns true if this is the empty (or root) path.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn strip_prefix(&self, prefix: &WorkPath) -> Option<&Self> {
        if prefix.is_empty() {
            Some(self)
        }
        else if let Some(suffix) = self.0.strip_prefix(&prefix.0) {
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
            else if suffix.len() == self.0.len() {
                Some(self)
            }
            else {
                unsafe { Some(Self::from_str(&suffix[..suffix.len() - 1])) }
            }
        }
        else {
            None
        }
    }

    /// Returns the path to the directory that contains this path.
    /// If this path is the root directory, `None` is returned.
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

    /// Returns the name of the file at this path (in other words, the final component).
    /// If this path is the root directory, the empty path is returned.
    pub fn file_name(&self) -> &Self {
        if let Some(last_sep_idx) = self.0.rfind('/') {
            let slice = &self.0[last_sep_idx + 1..];
            unsafe { Self::from_str(slice) }
        }
        else {
            self
        }
    }

    /// Splits the path between its first and second components.
    /// If there is only one component, the second element of the tuple will be `None`.
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

impl AsRef<Path> for WorkPath {
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl ToOwned for WorkPath {
    type Owned = WorkPathBuf;
    
    fn to_owned(&self) -> Self::Owned {
        Self::Owned::from(self)
    }
}

impl PartialEq<str> for WorkPath {
    fn eq(&self, other: &str) -> bool {
        &self.0 == other
    }
}

impl fmt::Display for WorkPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl WorkPathBuf {
    /// Creates a new `WorkPathBuf` from the empty (or root) path.
    pub fn root() -> Self {
        Self("".to_owned())
    }

    /// Concatenates `path` to end of this path.
    pub fn push(&mut self, path: &WorkPath) {
        if !self.0.is_empty() {
            self.0.push('/');
        }
        self.0.push_str(&path.0);
    }

    /// Removes the last component of this path, if any. Returns `true` if a component was removed.
    pub fn pop(&mut self) -> bool {
        if self.0.is_empty() {
            return false;
        }
        
        let last_sep_index = self.0.rfind('/').unwrap_or(0);
        self.0.truncate(last_sep_index);

        true
    }

    /// Creates a new path by concatenating `path` to the end of this path.
    pub fn join(&self, path: &WorkPath) -> WorkPathBuf {
        let mut new_path = self.clone();
        new_path.push(path);
        new_path
    }

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
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let path = value.replace('\\', "/");

        if path.starts_with('/') || path.contains(':') {
            return Err(WorkDirError::AbsolutePath(PathBuf::from(value)).into());
        }

        let normalized_path =
            path.split('/')
            .filter_map(|part| {
                if part.is_empty() {
                    None
                }
                else if [".", "..", ".git"].contains(&part) {
                    Some(Err(WorkDirError::ForbiddenComponent {
                        path: PathBuf::from(value),
                        component: part.to_owned(),
                    }))
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
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::try_from(value.as_str())
    }
}

impl TryFrom<OsString> for WorkPathBuf {
    type Error = anyhow::Error;

    fn try_from(value: OsString) -> Result<Self, Self::Error> {
        if let Some(value) = value.to_str() {
            Self::try_from(value)
        }
        else {
            Err(WorkDirError::InvalidUnicode(value).into())
        }
    }
}

impl TryFrom<&Path> for WorkPathBuf {
    type Error = anyhow::Error;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if let Some(path) = value.to_str() {
            Self::try_from(path)
        }
        else {
            Err(WorkDirError::InvalidUnicode(value.as_os_str().to_owned()).into())
        }
    }
}

impl TryFrom<PathBuf> for WorkPathBuf {
    type Error = anyhow::Error;

    fn try_from(value: PathBuf) -> Result<Self, Self::Error> {
        Self::try_from(value.as_path())
    }
}

impl TryFrom<&[u8]> for WorkPathBuf {
    type Error = anyhow::Error;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slashes_are_normalized() {
        {
            let path = WorkPathBuf::try_from(r"this/is\my/mixed\slash\path").unwrap();
            assert_eq!(path.as_str(), "this/is/my/mixed/slash/path");
        }
        {
            let path = WorkPathBuf::try_from(r"this/\path//has\\repeated\/slashes").unwrap();
            assert_eq!(path.as_str(), "this/path/has/repeated/slashes");
        }
        {
            let path = WorkPathBuf::try_from(r"trailing/slash/").unwrap();
            assert_eq!(path.as_str(), "trailing/slash");
        }
    }

    #[test]
    fn absolute_paths_are_rejected() {
        {
            let result = WorkPathBuf::try_from("/this/is/my/absolute/path");
            assert!(result.is_err());
        }
        {
            let result = WorkPathBuf::try_from(r"C:\this\my\absolute\path\on\windows");
            assert!(result.is_err());
        }
        {
            let result = WorkPathBuf::try_from(r"\\this\is\my\windows\network\path");
            assert!(result.is_err());
        }
    }

    #[test]
    fn forbidden_components_are_rejected() {
        {
            let result = WorkPathBuf::try_from(".");
            assert!(result.is_err());
        }
        {
            let result = WorkPathBuf::try_from(r"my/../path");
            assert!(result.is_err());
        }
        {
            let result = WorkPathBuf::try_from(r"path/to/.git/directory");
            assert!(result.is_err());
        }
    }

    #[test]
    fn push_to_empty_path() {
        let mut path = WorkPathBuf::try_from("").unwrap();
        let subpath = WorkPathBuf::try_from("hello").unwrap();
        path.push(&subpath);
        assert_eq!(path.as_str(), "hello");
    }

    #[test]
    fn push_to_nonempty_path() {
        let mut path = WorkPathBuf::try_from("hello").unwrap();
        let subpath = WorkPathBuf::try_from("world/good/morning").unwrap();
        path.push(&subpath);
        assert_eq!(path.as_str(), "hello/world/good/morning");
    }

    #[test]
    fn pop_from_empty_path() {
        let mut path = WorkPathBuf::try_from("").unwrap();
        let was_popped = path.pop();
        assert!(!was_popped);
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn pop_from_single_component_path() {
        let mut path = WorkPathBuf::try_from("hello").unwrap();
        let was_popped = path.pop();
        assert!(was_popped);
        assert_eq!(path.as_str(), "");
    }

    #[test]
    fn pop_from_multi_component_path() {
        let mut path = WorkPathBuf::try_from("hello/world").unwrap();
        let was_popped = path.pop();
        assert!(was_popped);
        assert_eq!(path.as_str(), "hello");
    }

    #[test]
    fn strip_prefix_not_present() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/world").unwrap();
        let prefix: &WorkPath = &WorkPathBuf::try_from("ahoy").unwrap();
        let suffix = path.strip_prefix(prefix);
        assert!(suffix.is_none());
    }

    #[test]
    fn strip_prefix_present() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
        {
            let prefix: &WorkPath = &WorkPathBuf::try_from("").unwrap();
            let suffix = path.strip_prefix(prefix).unwrap();
            assert_eq!(suffix, "hello/there/world");
        }
        {
            let prefix: &WorkPath = &WorkPathBuf::try_from("hello").unwrap();
            let suffix = path.strip_prefix(prefix).unwrap();
            assert_eq!(suffix, "there/world");
        }
        {
            let prefix: &WorkPath = &WorkPathBuf::try_from("hello/there").unwrap();
            let suffix = path.strip_prefix(prefix).unwrap();
            assert_eq!(suffix, "world");
        }
        {
            let prefix: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
            let suffix = path.strip_prefix(prefix).unwrap();
            assert_eq!(suffix, "");
        }
    }

    #[test]
    fn strip_suffix_not_present() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/world").unwrap();
        let suffix: &WorkPath = &WorkPathBuf::try_from("earth").unwrap();
        let prefix = path.strip_suffix(suffix);
        assert!(prefix.is_none());
    }

    #[test]
    fn strip_suffix_present() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
        {
            let suffix: &WorkPath = &WorkPathBuf::try_from("").unwrap();
            let prefix = path.strip_suffix(suffix).unwrap();
            assert_eq!(prefix, "hello/there/world");
        }
        {
            let suffix: &WorkPath = &WorkPathBuf::try_from("world").unwrap();
            let prefix = path.strip_suffix(suffix).unwrap();
            assert_eq!(prefix, "hello/there");
        }
        {
            let suffix: &WorkPath = &WorkPathBuf::try_from("there/world").unwrap();
            let prefix = path.strip_suffix(suffix).unwrap();
            assert_eq!(prefix, "hello");
        }
        {
            let suffix: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
            let prefix = path.strip_suffix(suffix).unwrap();
            assert_eq!(prefix, "");
        }
    }

    #[test]
    fn parent_of_empty_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("").unwrap();
        let parent = path.parent();
        assert!(parent.is_none());
    }

    #[test]
    fn parent_of_single_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent, "");
    }

    #[test]
    fn parent_of_multi_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
        let parent = path.parent().unwrap();
        assert_eq!(parent, "hello/there");
    }

    #[test]
    fn file_name_of_empty_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("").unwrap();
        let file_name = path.file_name();
        assert_eq!(file_name, "");
    }

    #[test]
    fn file_name_of_single_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello").unwrap();
        let file_name = path.file_name();
        assert_eq!(file_name, "hello");
    }

    #[test]
    fn file_name_of_multi_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
        let file_name = path.file_name();
        assert_eq!(file_name, "world");
    }

    #[test]
    fn partition_empty_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("").unwrap();
        let (first, rest) = path.partition();
        assert_eq!(first, "");
        assert!(rest.is_none());
    }

    #[test]
    fn partition_single_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello").unwrap();
        let (first, rest) = path.partition();
        assert_eq!(first, "hello");
        assert!(rest.is_none());
    }

    #[test]
    fn partition_multi_component_path() {
        let path: &WorkPath = &WorkPathBuf::try_from("hello/there/world").unwrap();
        let (first, rest) = path.partition();
        assert_eq!(first, "hello");
        assert_eq!(rest.unwrap(), "there/world");
    }
}
