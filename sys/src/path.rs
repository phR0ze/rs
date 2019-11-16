use glob::glob;
use std::env;
use std::io;
use std::path::{Component, Path, PathBuf};

use errors::Result;

// Path utilities
// -------------------------------------------------------------------------------------------------
pub fn abs<T: AsRef<Path>>(path: T) -> Result<PathBuf> {
    let _path = path.as_ref();

    // Check for empty string
    if _path.to_string()? == "" {
        return Err(Box::from(io::Error::new(io::ErrorKind::Other, "Empty string is an invalid path")));
    }

    // Expand home directory and trim trailing slash if needed
    let mut path_buf = _path.expand()?;
    let mut path_str = path_buf.to_string()?;
    if path_str.len() > 1 {
        path_buf = path_buf.trim_end_matches("/")?;
        path_str = path_buf.to_string()?;
    }

    // Expand current directory if needed
    if !path_buf.is_absolute() {
        // Unwrap is acceptable here as Some will always exist
        match path_str.split("/").next().unwrap() {
            "." => path_buf = env::current_dir()?.join(&path_str[1..]),
            ".." => path_buf = env::current_dir()?.dirname()?.join(&path_str[2..]),
            _ => path_buf = env::current_dir()?.join(path_buf),
        }
    }

    // Clean the path

    Ok(path_buf)
}

// Returns the full path to the directory of the current running executable.
pub fn exec_dir() -> Result<PathBuf> {
    Ok(env::current_exe()?.dirname()?)
}

// Returns the current running executable's name.
pub fn exec_name() -> Result<String> {
    Ok(env::current_exe()?.name()?)
}

// Returns a vector of all paths from the given target glob, sorted by name.
// Doesn't include the target itself only its children nor is this recursive.
pub fn getpaths<T: AsRef<Path>>(pattern: T) -> Result<Vec<PathBuf>> {
    let mut paths: Vec<PathBuf> = Vec::new();
    let _str = pattern.as_ref().to_string()?;
    for x in glob(&_str)? {
        let path = x?;
        paths.push(path);
    }
    Ok(paths)
}

// Path extensions
// -------------------------------------------------------------------------------------------------
pub trait PathExt {
    fn contains_str<T: AsRef<str>>(&self, value: T) -> bool;
    fn clean(&self) -> Result<PathBuf>;
    fn dirname(&self) -> Result<PathBuf>;
    fn empty(&self) -> bool;
    fn expand(&self) -> Result<PathBuf>;
    fn name(&self) -> Result<String>;
    fn starts_with_str<T: AsRef<str>>(&self, value: T) -> bool;
    fn to_string(&self) -> Result<String>;
    fn trim_protocol(&self) -> Result<PathBuf>;
    fn trim_end_matches<T: AsRef<str>>(&self, value: T) -> Result<PathBuf>;
}
impl PathExt for Path {
    // Returns true if the `Path` as a String contains the given string
    fn contains_str<T: AsRef<str>>(&self, value: T) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        if _str.contains(value.as_ref()) {
            return true;
        }
        false
    }

    // Return the shortest path equivalent to the path by purely lexical processing and thus does not handle
    // links correctly in some cases, use canonicalize in those cases. It applies the following rules
    // interatively until no further processing can be done.
    //
    //	1. Replace multiple slashes with a single
    //	2. Eliminate each . path name element (the current directory)
    //	3. Eliminate each inner .. path name element (the parent directory)
    //	   along with the non-.. element that precedes it.
    //	4. Eliminate .. elements that begin a rooted path:
    //	   that is, replace "/.." by "/" at the beginning of a path.
    //  5. Leave intact ".." elements that begin a non-rooted path.
    //  6. Drop trailing '/' unless it is the root
    //
    // If the result of this process is an empty string, return the string `.`, representing the current directory.
    fn clean(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;

        // Components already handles the following cases:
        // 1. Repeated separators are ignored, so a/b and a//b both have a and b as components.
        // 2. Occurrences of . are normalized away, except if they are at the beginning of the path.
        //    e.g. a/./b, a/b/, a/b/. and a/b all have a and b as components, but ./a/b starts with an additional CurDir component.
        // 6. A trailing slash is normalized away, /a/b and /a/b/ are equivalent.
        let mut cnt = 0;
        let mut path_buf = PathBuf::new();
        for component in self.components() {
            if component == Component::ParentDir {
                // } && cnt > 0 => (),
                //     _ => (),
            }
            cnt += 1;
            path_buf.push(component);
        }

        // Ensure if empty the current dir is returned
        if path_buf.empty() {
            path_buf.push(".");
        }
        Ok(path_buf)
    }

    // Returns the `Path` without its final component, if there is one.
    fn dirname(&self) -> Result<PathBuf> {
        let dir = self.parent().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Parent directory not found"))?;
        Ok(dir.to_path_buf())
    }

    // Expand the path to include the home prefix if necessary
    fn expand(&self) -> Result<PathBuf> {
        let path_str = self.to_string()?;
        let mut expanded = self.to_path_buf();

        // Check for invalid home expansion
        match path_str.matches("~").count() {
            // Only home expansion at the begining of the path is allowed
            cnt if cnt > 1 => return Err(Box::from(io::Error::new(io::ErrorKind::Other, "Only one tilda is allowed"))),

            // Invalid home expansion requested
            cnt if cnt == 1 && !self.starts_with_str("~/") => {
                return Err(Box::from(io::Error::new(io::ErrorKind::Other, "Invalid home expansion requested")))
            }

            // Replace prefix with home directory
            1 => expanded = crate::user_home()?.join(&path_str[2..]),
            _ => (),
        }

        Ok(expanded)
    }

    // Returns true if the `Path` is empty.
    fn empty(&self) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        _str == ""
    }

    // Returns the final component of the `Path`, if there is one.
    fn name(&self) -> Result<String> {
        let os_str = self.file_name().ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Filename not found"))?;
        let filename = os_str.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to convert filename into str"))?;
        Ok(String::from(filename))
    }

    // Returns true if the `Path` as a String starts with the given string
    fn starts_with_str<T: AsRef<str>>(&self, value: T) -> bool {
        let res = self.to_string();
        let _str = match res {
            Ok(s) => s,
            Err(_) => return false,
        };
        if _str.contains(value.as_ref()) {
            return true;
        }
        false
    }

    // Returns the `Path` as a String
    fn to_string(&self) -> Result<String> {
        let _str = self.to_str().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Unable to convert Path into String"))?;
        Ok(String::from(_str))
    }

    // Returns the `Path` with well known protocol prefixes removed.
    fn trim_protocol(&self) -> Result<PathBuf> {
        let _str = self.to_string()?;
        let _str = _str.to_lowercase();
        let _str = _str.trim_start_matches("file://");
        let _str = _str.trim_start_matches("ftp://");
        let _str = _str.trim_start_matches("http://");
        let _str = _str.trim_start_matches("https://");
        Ok(PathBuf::from(_str))
    }

    // Returns a string slice with all suffixes that match a pattern repeatedly removed.
    fn trim_end_matches<T: AsRef<str>>(&self, value: T) -> Result<PathBuf> {
        let _str = self.to_string()?;
        let _value = value.as_ref();
        Ok(PathBuf::from(_str.trim_end_matches(_value)))
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_abs() {
        let home = env::var("HOME").unwrap();
        let cwd = env::current_dir().unwrap();
        let prev = cwd.dirname().unwrap();

        // expand previous directory and drop slash
        assert_eq!(PathBuf::from(&prev), abs("../").unwrap());

        // expand previous directory
        assert_eq!(PathBuf::from(&prev), abs("..").unwrap());

        // expand current directory
        assert_eq!(PathBuf::from(&cwd), abs(".").unwrap());

        // expand relative directory
        assert_eq!(PathBuf::from(&cwd).join("foo"), abs("foo").unwrap());

        // expand home path
        assert_eq!(PathBuf::from(&home).join("foo"), abs("~/foo").unwrap());
    }

    #[test]
    fn test_exec_dir() {
        let cwd = env::current_dir().unwrap();
        let dir = cwd.parent().unwrap().join("target/debug/deps");
        assert_eq!(dir, exec_dir().unwrap());
    }

    #[test]
    fn test_exec_name() {
        let exec_path = env::current_exe().unwrap();
        let name = exec_path.name().unwrap();
        assert_eq!(name, exec_name().unwrap());
    }

    #[test]
    fn test_getpaths() {
        let paths = getpaths(&"*").unwrap();
        assert_eq!(&PathBuf::from(".vscode"), paths.first().unwrap());
        assert_eq!(&PathBuf::from("src"), paths.last().unwrap());
    }

    // Path tests
    // ---------------------------------------------------------------------------------------------
    #[test]
    fn test_pathext_contains() {
        let path = PathBuf::from("/foo/bar");
        assert!(path.contains_str("foo"));
        assert!(path.contains_str("/foo"));
        assert!(path.contains_str("/"));
        assert!(path.contains_str("/ba"));
        assert!(!path.contains_str("bob"));
    }

    #[test]
    fn test_pathext_clean() {
        // remove uneeded double slashes
        {
            let path = PathBuf::from("/foo//bar");
            assert_eq!(PathBuf::from("/foo/bar"), path.clean().unwrap());
        }
    }

    #[test]
    fn test_pathext_dirname() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!(PathBuf::from("/foo").as_path(), path.dirname().unwrap());
    }

    #[test]
    fn test_pathext_empty() {
        // empty string
        {
            let path = PathBuf::from("");
            assert!(path.empty());
        }

        // false
        {
            let path = PathBuf::from("/foo");
            assert!(!path.empty());
        }
    }

    #[test]
    fn test_pathext_expand() {
        // happy path
        {
            let home = env::var("HOME").unwrap();
            let path = PathBuf::from("~/foo");
            assert_eq!(PathBuf::from(&home).join("foo"), path.expand().unwrap());
        }

        // More than one ~
        {
            let path = PathBuf::from("~/foo~");
            assert!(path.expand().is_err());
        }

        // invalid path
        {
            let path = PathBuf::from("~foo");
            assert!(path.expand().is_err());
        }

        // empty path - nothing to do but no error
        {
            let path = PathBuf::from("");
            assert_eq!(PathBuf::from(""), path.expand().unwrap());
        }

        // home not set
        {
            let save = env::var("HOME").unwrap();
            env::remove_var("HOME");
            let path = PathBuf::from("~/foo");
            assert!(path.expand().is_err());
            env::set_var("HOME", &save);
        }
    }

    #[test]
    fn test_pathext_filename() {
        let path = PathBuf::from("/foo/bar");
        assert_eq!("bar", path.name().unwrap());
    }

    #[test]
    fn test_pathext_to_string() {
        let path = PathBuf::from("/foo");
        assert_eq!("/foo".to_string(), path.to_string().unwrap());
    }

    #[test]
    fn test_pathext_trim_protocol() {
        // no change
        {
            let path = PathBuf::from("/foo");
            assert_eq!(PathBuf::from("/foo"), path.trim_protocol().unwrap());
        }
        // file://
        {
            let path = PathBuf::from("file:///foo");
            assert_eq!(PathBuf::from("/foo"), path.trim_protocol().unwrap());
        }
        // ftp://
        {
            let path = PathBuf::from("ftp://foo");
            assert_eq!(PathBuf::from("foo"), path.trim_protocol().unwrap());
        }
        // http://
        {
            let path = PathBuf::from("http://foo");
            assert_eq!(PathBuf::from("foo"), path.trim_protocol().unwrap());
        }
        // https://
        {
            let path = PathBuf::from("https://foo");
            assert_eq!(PathBuf::from("foo"), path.trim_protocol().unwrap());
        }
        // HTTPS://
        {
            let path = PathBuf::from("HTTPS://foo");
            assert_eq!(PathBuf::from("foo"), path.trim_protocol().unwrap());
        }
    }

    #[test]
    fn test_pathext_trim_end_matches() {
        // drop root
        {
            let path = PathBuf::from("/");
            assert_eq!(PathBuf::new(), path.trim_end_matches("/").unwrap());
        }

        // drop end
        {
            let path = PathBuf::from("/foo/");
            assert_eq!(PathBuf::from("/foo"), path.trim_end_matches("/").unwrap());
        }

        // no change
        {
            let path = PathBuf::from("/foo");
            assert_eq!(PathBuf::from("/foo"), path.trim_end_matches("/").unwrap());
        }
    }
}
