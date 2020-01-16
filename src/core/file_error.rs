use failure::Fail;
use std::fmt;

// An error indicating that something went wrong with a file operation
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Fail)]
pub enum FileError {
    /// An error indicating that a regex string extraction failed.
    FailedToExtractString,
}

impl fmt::Display for FileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            FileError::FailedToExtractString => write!(f, "failed to extract string from file"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::core::*;

    #[test]
    fn test_errors() {
        assert_eq!(format!("{}", FileError::FailedToExtractString), "failed to extract string from file");
    }
}
