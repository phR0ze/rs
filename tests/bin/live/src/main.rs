use fungus::prelude::*;

fn main() -> Result<()> {
    println!("=====================================================================");
    println!("=                         LIVE TESTING                              =");
    println!("=====================================================================");
    println!("HOME: {:?}", user::home_dir().unwrap());
    test_git()?;
    test_libc()?;
    test_crypto()?;
    Ok(())
}

// fn test_arch<T: AsRef<Path>>(path: T) {
//     let tmpdir = setup(path, "bin_net");

//     assert_eq!(abs::repo("acme").unwrap(), abs::Repo::Community);

//     cleanup(tmpdir);
// }

fn test_git() -> Result<()> {
    assert_eq!(net::git::remote_branch_exists("https://github.com/phR0ze/fungus.git", "master"), true);
    //assert_eq!(net::git::remote_branch_exists("https://git.archlinux.org/svntogit/packages.git", "pkgfile"), true);
    println!("Git tests passed!");
    Ok(())
}

fn test_libc() -> Result<()> {
    println!("User ID: {}", user::getuid());
    Ok(())
}

fn test_crypto() -> Result<()> {
    let tmpfile = Path::new("~/.bashrc").abs()?;
    println!("Digest: {}", enc::hex::encode(sys::digest(&tmpfile)?));
    Ok(())
}
