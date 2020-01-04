#[macro_use]
pub mod core;
pub mod net;
pub mod sys;

/// Import all the modules in a consumable way
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub mod prelude {
    pub use super::core::*;
    pub use super::net::{self, agent};
    pub use super::sys::{self, exec, ext::*, user};
    pub use super::{cfgblock, fatal};

    // Re-exports
    //----------------------------------------------------------------------------------------------
    pub use chrono;
    pub use colored::*;
    pub use lazy_static::*;
    pub use log;
    pub use regex::Regex;
    pub use std::env;
    pub use std::ffi::OsStr;
    pub use std::fmt;
    pub use std::fs::{self, File, OpenOptions};
    pub use std::io::{self, prelude::*, BufRead, BufReader};
    pub use std::os::unix::fs::{MetadataExt, PermissionsExt};
    pub use std::path::{Component, Path, PathBuf};
    pub use std::str;
}
