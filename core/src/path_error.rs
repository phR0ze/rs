use std::fmt;
use failure::Fail;
use std::path::{Path, PathBuf};

// An error indicating that something went wrong with a path operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum PathError {
    /// An error indicating that the path is empty.
    Empty,

    /// An error indicating a failure to convert the path to a string.
    FailedToString(PathBuf),

    /// An error indicating that the path does not contain a filename.
    FileNameNotFound(PathBuf),

    /// An error indicating that the path failed to expand properly.
    InvalidExpansion(PathBuf),

    /// An error indicating that the path contains multiple user home symbols i.e. tilda.
    MultipleHomeSymbols(PathBuf),

    /// An error indicating that the path does not have a valid parent path.
    ParentNotFound(PathBuf),
}
impl PathError {
    /// Return an error indicating that the path is empty
    pub fn empty() -> PathError {
        PathError::Empty
    }

    /// Return an error indicating a failure to convert the path to a string
    pub fn failed_to_string<T: AsRef<Path>>(path: T) -> PathError {
        PathError::FailedToString(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path does not contain a filename
    pub fn filename_not_found<T: AsRef<Path>>(path: T) -> PathError {
        PathError::FileNameNotFound(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path failed to expand properly
    pub fn invalid_expansion<T: AsRef<Path>>(path: T) -> PathError {
        PathError::InvalidExpansion(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path contains multiple user home symbols i.e. tilda
    pub fn multiple_home_symbols<T: AsRef<Path>>(path: T) -> PathError {
        PathError::MultipleHomeSymbols(path.as_ref().to_path_buf())
    }

    /// Return an error indicating that the path does not have a valid parent path
    pub fn parent_not_found<T: AsRef<Path>>(path: T) -> PathError {
        PathError::ParentNotFound(path.as_ref().to_path_buf())
    }
}

impl fmt::Display for PathError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            PathError::Empty => write!(f, "path empty"),
            PathError::FailedToString(ref path) => write!(f, "failed to convert to string for path {}", path.display()),
            PathError::FileNameNotFound(ref path) => write!(f, "filename not found for path {}", path.display()),
            PathError::InvalidExpansion(ref path) => write!(f, "invalid path expansion for path {}", path.display()),
            PathError::MultipleHomeSymbols(ref path) => write!(f, "multiple home symbols for path {}", path.display()),
            PathError::ParentNotFound(ref path) => write!(f, "parent not found for path {}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::*;

    fn path_empty() -> Result<PathBuf> {
        Err(PathError::empty().into())
    }

    fn parent_not_found() -> Result<PathBuf> {
        Err(PathError::parent_not_found("foo").into())
    }

    #[test]
    fn test_new_path_empty() {
        assert!(path_empty().is_err());
        assert_eq!(path_empty().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::Empty));
    }

    #[test]
    fn test_parent_not_found() {
        assert!(parent_not_found().is_err());
        assert_ne!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("bar")));
        assert_eq!(parent_not_found().unwrap_err().downcast_ref::<PathError>(), Some(&PathError::parent_not_found("foo")));
        assert_eq!(format!("{}", parent_not_found().unwrap_err().downcast_ref::<PathError>().unwrap()), "parent not found for path foo");
    }

    #[test]
    fn test_backtrace() {
        let err = path_empty().unwrap_err();
        println!("{:?}", err);
    }
}