use std::ffi::OsStr;
use std::path::Path;
use std::str;

use crate::core::*;

pub trait StringSizeExt {
    /// Returns the length in characters rather than bytes i.e. this is a human understandable
    /// value. However it is more costly to perform.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!("foo".size(), 3);
    /// assert_eq!("ƒoo".len(), 4); // fancy f!
    /// assert_eq!("ƒoo".size(), 3); // fancy f!
    /// ```
    fn size(&self) -> usize;
}
impl StringSizeExt for str {
    fn size(&self) -> usize {
        self.chars().count()
    }
}

impl StringSizeExt for String {
    fn size(&self) -> usize {
        self.chars().count()
    }
}

pub trait ToStringExt {
    /// Returns a new [`String`] from the given type.
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(OsStr::new("foo").to_string().unwrap(), "foo".to_string());
    /// assert_eq!(Path::new("/foo").to_string().unwrap(), "/foo".to_string());
    /// ```
    fn to_string(&self) -> Result<String>;
}

impl ToStringExt for Path {
    fn to_string(&self) -> Result<String> {
        let _str = self.to_str().ok_or_else(|| PathError::failed_to_string(self))?;
        Ok(String::from(_str))
    }
}

impl ToStringExt for OsStr {
    fn to_string(&self) -> Result<String> {
        Ok(String::from(self.to_str().ok_or_else(|| StringError::FailedToString)?))
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_str_size() {
        assert_eq!("foo".size(), 3);
        assert_eq!("ƒoo".len(), 4); // fancy f!
        assert_eq!("ƒoo".size(), 3); // fancy f!
    }

    #[test]
    fn test_string_size() {
        assert_eq!("foo".to_string().size(), 3);
        assert_eq!("ƒoo".to_string().len(), 4); // fancy f!
        assert_eq!("ƒoo".to_string().size(), 3); // fancy f!
    }

    #[test]
    fn test_osstr_to_string() {
        assert_eq!(OsStr::new("foo").to_string().unwrap(), "foo".to_string());
    }

    #[test]
    fn test_path_to_string() {
        assert_eq!(Path::new("/foo").to_string().unwrap(), "/foo".to_string());
        assert_eq!(PathBuf::from("/foo").to_string().unwrap(), "/foo".to_string());
    }
}
