cfgblock! {
    #[cfg(feature = "_libc_")]
    use libc;
    use std::io;
    use std::mem;
    use std::ptr;
}
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::env;
use std::path::PathBuf;

use crate::prelude::*;

// Implementation in Rust for the XDB Base Directory Specification
// https://wiki.archlinux.org/index.php/XDG_Base_Directory
// -------------------------------------------------------------------------------------------------

/// Returns the full path to the current user's home directory.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::home_dir().is_ok());
/// ```
pub fn home_dir() -> Result<PathBuf> {
    let os_str = env::var("HOME")?;
    let dir = PathBuf::from(os_str);
    Ok(dir)
}

/// Returns the full path to the current user's config directory.
/// Where user-specific configurations should be written (analogous to /etc).
/// Defaults to $HOME/.config.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::config_dir().is_ok());
/// ```
pub fn config_dir() -> Result<PathBuf> {
    Ok(match env::var("XDG_CONFIG_HOME") {
        Ok(x) => PathBuf::from(x),
        Err(_) => home_dir()?.mash(".config"),
    })
}

/// Returns the full path to the current user's cache directory.
/// Where user-specific non-essential (cached) data should be written (analogous to /var/cache).
/// Defaults to $HOME/.cache.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::cache_dir().is_ok());
/// ```
pub fn cache_dir() -> Result<PathBuf> {
    Ok(match env::var("XDG_CACHE_HOME") {
        Ok(x) => PathBuf::from(x),
        Err(_) => home_dir()?.mash(".cache"),
    })
}

/// Returns the full path to the current user's data directory.
/// Where user-specific data files should be written (analogous to /usr/share).
/// Defaults to $HOME/.local/share
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::data_dir().is_ok());
/// ```
pub fn data_dir() -> Result<PathBuf> {
    Ok(match env::var("XDG_DATA_HOME") {
        Ok(x) => PathBuf::from(x),
        Err(_) => home_dir()?.mash(".local/share"),
    })
}

/// Returns the full path to the current user's runtime directory.
/// Used for non-essential, user-specific data files such as sockets, named pipes, etc.
/// Must be owned by the user with an access mode of 0700.
/// Filesystem fully featured by standards of OS.
/// Must be on the local filesystem.
/// May be subject to periodic cleanup.
/// Modified every 6 hours or set sticky bit if persistence is desired.
/// Can only exist for the duration of the user's login.
/// Should not store large files as it may be mounted as a tmpfs.
///
/// Defaults to /tmp if $XDG_RUNTIME_DIR is not set
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// println!("runtime directory of the current user: {:?}", user::runtime_dir());
/// ```
pub fn runtime_dir() -> PathBuf {
    match env::var("XDG_RUNTIME_DIR") {
        Ok(x) => PathBuf::from(x),
        Err(_) => PathBuf::from("/tmp"),
    }
}

/// Returns the full path to a newly created directory in `/tmp` that can be used for temporary
/// work. The returned path will be checked for uniqueness and created with a random suffix and
/// the given `prefix`. It is up to the calling code to ensure the directory returned is
/// properly cleaned up when done with.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// let tmpdir = user::temp_dir("foo").unwrap();
/// assert_eq!(tmpdir.exists(), true);
/// {
///     let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
/// }
/// assert_eq!(tmpdir.exists(), false);
/// ```
pub fn temp_dir<T: AsRef<str>>(prefix: T) -> Result<PathBuf> {
    loop {
        let suffix: String = rand::thread_rng().sample_iter(&Alphanumeric).take(8).collect();
        let dir = PathBuf::from(format!("/tmp/{}-{}", prefix.as_ref(), suffix));
        if !dir.exists() {
            return sys::mkdir(&dir);
        }
    }
}

/// Returns the current user's data directories.
/// List of directories seperated by : (analogous to PATH).
/// Defaults to /usr/local/share:/usr/share.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::data_dirs().is_ok());
/// ```
pub fn data_dirs() -> Result<Vec<PathBuf>> {
    Ok(match env::var("XDG_DATA_DIRS") {
        Ok(x) => sys::parse_paths(x)?,
        Err(_) => vec![PathBuf::from("/usr/local/share:/usr/share")],
    })
}

/// Returns the current user's config directories.
/// List of directories seperated by : (analogous to PATH).
/// Defaults to /etc/xdg
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::config_dirs().is_ok());
/// ```
pub fn config_dirs() -> Result<Vec<PathBuf>> {
    Ok(match env::var("XDG_CONFIG_DIRS") {
        Ok(x) => sys::parse_paths(x)?,
        Err(_) => vec![PathBuf::from("/etc/xdg")],
    })
}

/// Returns the current user's path directories.
/// List of directories seperated by :
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::path_dirs().is_ok());
/// ```
pub fn path_dirs() -> Result<Vec<PathBuf>> {
    sys::parse_paths(env::var("PATH")?)
}

// User functions
// -------------------------------------------------------------------------------------------------

/// User provides options for a specific user.
#[cfg(feature = "_libc_")]
#[derive(Debug, Clone, Default)]
pub struct User {
    pub uid: u32,           // user id
    pub gid: u32,           // user group id
    pub name: String,       // user name
    pub home: PathBuf,      // user home
    pub shell: PathBuf,     // user shell
    pub ruid: u32,          // real user id behind sudo
    pub rgid: u32,          // real user group id behind sudo
    pub realname: String,   // real user name behind sudo
    pub realhome: PathBuf,  // real user home behind sudo
    pub realshell: PathBuf, // real user shell behind sudo
}

#[cfg(feature = "_libc_")]
impl User {
    /// Returns true if the user is root
    ///
    /// ### Examples
    /// ```
    /// use fungus::prelude::*;
    ///
    /// assert_eq!(user::current().unwrap().is_root(), false);
    /// ```
    pub fn is_root(&self) -> bool {
        self.uid == 0
    }
}

/// Get the current user
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::current().is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn current() -> Result<User> {
    let user = lookup(unsafe { libc::getuid() })?;
    Ok(user)
}

/// Switches back to the original user under the sudo mask with no way to go back.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::drop_sudo().is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn drop_sudo() -> Result<()> {
    match getuid() {
        0 => {
            let (ruid, rgid) = getrids(0, 0);
            switchuser(ruid, ruid, ruid, rgid, rgid, rgid)
        }
        _ => Ok(()),
    }
}

/// Returns the user ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::getuid() != 0);
/// ```
#[cfg(feature = "_libc_")]
pub fn getuid() -> u32 {
    unsafe { libc::getuid() }
}

/// Returns the group ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::getgid() != 0);
/// ```
#[cfg(feature = "_libc_")]
pub fn getgid() -> u32 {
    unsafe { libc::getgid() }
}

/// Returns the user effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::geteuid() != 0);
/// ```
#[cfg(feature = "_libc_")]
pub fn geteuid() -> u32 {
    unsafe { libc::geteuid() }
}

/// Returns the group effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::getegid() != 0);
/// ```
#[cfg(feature = "_libc_")]
pub fn getegid() -> u32 {
    unsafe { libc::getegid() }
}

/// Returns the real IDs for the given user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(user::getrids(user::getuid(), user::getgid()), (user::getuid(), user::getgid()));
/// ```
#[cfg(feature = "_libc_")]
pub fn getrids(uid: u32, gid: u32) -> (u32, u32) {
    match uid {
        0 => match (env::var("SUDO_UID"), env::var("SUDO_GID")) {
            (Ok(u), Ok(g)) => match (u.parse::<u32>(), g.parse::<u32>()) {
                (Ok(u), Ok(g)) => (u, g),
                _ => (uid, gid),
            },
            _ => (uid, gid),
        },
        _ => (uid, gid),
    }
}

/// Return true if the current user is the root user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert_eq!(user::is_root(), false);
/// ```
#[cfg(feature = "_libc_")]
pub fn is_root() -> bool {
    getuid() == 0
}

/// Lookup a user by user id
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::lookup(user::getuid()).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn lookup(uid: u32) -> Result<User> {
    // Get the libc::passwd by user id
    let mut buf = vec![0; 2048];
    let mut res = ptr::null_mut::<libc::passwd>();
    let mut passwd = unsafe { mem::zeroed::<libc::passwd>() };
    unsafe {
        libc::getpwuid_r(uid, &mut passwd, buf.as_mut_ptr(), buf.len(), &mut res);
    }
    if res.is_null() || res != &mut passwd {
        return Err(UserError::does_not_exist_by_id(uid).into());
    }

    // Convert libc::passwd object into a User object
    //----------------------------------------------------------------------------------------------
    let gid = passwd.pw_gid;

    // User name for the lookedup user. We always want this and it should always exist.
    let username = unsafe { crate::sys::libc::to_string(passwd.pw_name)? };

    // Will almost always be a single 'x' as the passwd is in the shadow database
    //let userpwd = unsafe { crate::sys::libc::to_string(passwd.pw_passwd)? };

    // User home directory e.g. '/home/<user>'. Might be a null pointer indicating the system default should be used
    let userhome = unsafe { crate::sys::libc::to_string(passwd.pw_dir) }.unwrap_or_default();

    // User shell e.g. '/bin/bash'. Might be a null pointer indicating the system default should be used
    let usershell = unsafe { crate::sys::libc::to_string(passwd.pw_shell) }.unwrap_or_default();

    // A string container user contextual information, possibly real name or phone number.
    //let usergecos = unsafe { crate::sys::libc::to_string(passwd.pw_gecos)? };

    // Get the user's real ids as well if applicable
    let (ruid, rgid) = getrids(uid, gid);
    let realuser = if uid != ruid {
        lookup(ruid)?
    } else {
        User {
            uid: uid,
            gid: gid,
            name: username.to_string(),
            home: PathBuf::from(&userhome),
            shell: PathBuf::from(&usershell),
            ..Default::default()
        }
    };
    Ok(User {
        uid: uid,
        gid: gid,
        name: username.to_string(),
        home: PathBuf::from(&userhome),
        shell: PathBuf::from(&usershell),
        ruid: ruid,
        rgid: rgid,
        realname: realuser.name,
        realhome: realuser.home,
        realshell: realuser.shell,
    })
}

/// Returns the current user's name.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// println!("current user name: {:?}", user::name().unwrap());
/// ```
#[cfg(feature = "_libc_")]
pub fn name() -> Result<String> {
    Ok(current()?.name)
}

/// Switches back to the original user under the sudo mask. Preserves the ability to raise sudo
/// again.
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// assert!(user::pause_sudo().is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn pause_sudo() -> Result<()> {
    match getuid() {
        0 => {
            let (ruid, rgid) = getrids(0, 0);
            switchuser(ruid, ruid, 0, rgid, rgid, 0)
        }
        _ => Ok(()),
    }
}

/// Set the user ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::setuid(user::getuid()).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn setuid(uid: u32) -> Result<()> {
    match unsafe { libc::setuid(uid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the user effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::seteuid(user::geteuid()).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn seteuid(euid: u32) -> Result<()> {
    match unsafe { libc::seteuid(euid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the group ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::setgid(user::getgid()).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn setgid(gid: u32) -> Result<()> {
    match unsafe { libc::setgid(gid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Set the group effective ID for the current user.
///
/// ### Examples
/// ```
/// use fungus::prelude::*;
///
/// assert!(user::setegid(user::getegid()).is_ok());
/// ```
#[cfg(feature = "_libc_")]
pub fn setegid(egid: u32) -> Result<()> {
    match unsafe { libc::setegid(egid) } {
        0 => Ok(()),
        _ => Err(io::Error::last_os_error().into()),
    }
}

/// Switches back to sudo root. Returns and error if not allowed.
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// user:sudo().unwrap();
/// ```
#[cfg(feature = "_libc_")]
pub fn sudo() -> Result<()> {
    switchuser(0, 0, 0, 0, 0, 0)
}

/// Switches to another use by setting the real, effective and saved user and group ids.
///
/// ### Examples
/// ```ignore
/// use fungus::prelude::*;
///
/// // Switch to user 1000 but preserve root priviledeges to switch again
/// user::switchuser(1000, 1000, 0, 1000, 1000, 0);
///
/// // Switch to user 1000 and drop root priviledges permanantely
/// user::switchuser(1000, 1000, 1000, 1000, 1000, 1000);
/// ```
#[cfg(feature = "_libc_")]
pub fn switchuser(ruid: u32, euid: u32, suid: u32, rgid: u32, egid: u32, sgid: u32) -> Result<()> {
    // Best practice to drop the group first
    match unsafe { libc::setresgid(rgid, egid, sgid) } {
        0 => match unsafe { libc::setresuid(ruid, euid, suid) } {
            0 => Ok(()),
            _ => Err(io::Error::last_os_error().into()),
        },
        _ => Err(io::Error::last_os_error().into()),
    }
}

// Unit tests
// -------------------------------------------------------------------------------------------------
#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn test_user_home() {
        let home_str = env::var("HOME").unwrap();
        let home_path = PathBuf::from(home_str);
        let home_dir = home_path.parent().unwrap();
        assert_eq!(home_dir.to_path_buf(), user::home_dir().unwrap().dir().unwrap());
    }

    #[test]
    fn test_user_ids() {
        assert!(user::drop_sudo().is_ok());
        assert!(user::getuid() != 0);
        assert!(user::getgid() != 0);
        assert!(user::geteuid() != 0);
        assert!(user::getegid() != 0);
        assert_eq!(user::getrids(user::getuid(), user::getgid()), (user::getuid(), user::getgid()));
        assert_eq!(user::is_root(), false);
        assert!(user::lookup(user::getuid()).is_ok());
        assert!(user::name().unwrap() != "".to_string());
        assert!(user::setegid(user::getegid()).is_ok());
        assert!(user::setgid(user::getgid()).is_ok());
        assert!(user::seteuid(user::geteuid()).is_ok());
        assert!(user::setuid(user::getuid()).is_ok());
    }

    #[test]
    fn test_user_dirs() {
        assert!(user::home_dir().is_ok());
        assert!(user::config_dir().is_ok());
        assert!(user::cache_dir().is_ok());
        assert!(user::data_dir().is_ok());
        user::runtime_dir();
        assert!(user::data_dirs().is_ok());
        assert!(user::config_dirs().is_ok());
        assert!(user::path_dirs().is_ok());
        assert!(user::current().is_ok());
        assert_eq!(user::current().unwrap().is_root(), false);

        let tmpdir = user::temp_dir("test_user_dirs").unwrap();
        assert_eq!(tmpdir.exists(), true);
        {
            let _f = finally(|| sys::remove_all(&tmpdir).unwrap());
        }
        assert_eq!(tmpdir.exists(), false);
    }

    #[test]
    fn test_temp_dir() {
        let tmpdir = user::temp_dir("foo").unwrap();
        assert_eq!(tmpdir.exists(), true);
        assert!(sys::remove_all(&tmpdir).is_ok());
        assert_eq!(tmpdir.exists(), false);
    }
}
