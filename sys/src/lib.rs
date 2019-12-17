mod file;
mod path;
mod users;
pub use file::files::*;
pub use path::paths::*;

/// Import traits and other top level namespace entities.
///
/// ### Examples
/// ```
/// use rs::sys::preamble::*;
///
/// let home = env::var("HOME").unwrap();
/// assert_eq!(PathBuf::from(&home), sys::abs("~").unwrap());
/// ```
pub mod preamble {
    use super::*;
    pub use path::PathExt;
    pub use users::*;
}
