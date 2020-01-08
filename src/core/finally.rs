/// Implementing the missing finally pattern in Rust using an object's destructor.
pub struct Finally<T: FnOnce()> {
    func: Option<T>,
}

/// Destructor custom implementation for Finally
impl<T: FnOnce()> Drop for Finally<T> {
    fn drop(&mut self) {
        if let Some(func) = self.func.take() {
            func()
        }
    }
}

/// Create a Finally object that will execute the given `func` when its destructor is called.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = PathBuf::from("tests/temp").abs().unwrap().mash("core_finally_doc");
/// assert!(sys::mkdir(&tmpdir).is_ok());
///
/// // Create scope that will trigger finally destructor
/// {
///     let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
/// }
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn finally<T: FnOnce()>(func: T) -> Finally<T> {
    Finally { func: Some(func) }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    // Reusable teset setup
    struct Setup {
        temp: PathBuf,
    }
    impl Setup {
        fn init() -> Self {
            let setup = Self { temp: PathBuf::from("tests/temp").abs().unwrap() };
            sys::mkdir(&setup.temp).unwrap();
            setup
        }
    }

    #[test]
    fn test_finally() {
        let setup = Setup::init();
        let tmpdir = setup.temp.mash("core_finally");
        assert!(sys::mkdir(&tmpdir).is_ok());

        // Create scope that will trigger finally destructor
        {
            let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
        }

        assert_eq!(tmpdir.exists(), false);
    }
}
