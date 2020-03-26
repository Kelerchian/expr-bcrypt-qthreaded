use std::env;
use std::path::{PathBuf};
use std::io;

pub fn get_in_out_paths() -> io::Result<(PathBuf,PathBuf)> {
    let mut dir = env::current_dir()?;
    dir.push("data");
    let mut in_path = dir.clone();
    in_path.push("in");
    in_path.push("password-small.txt");

    let mut out_path = dir.clone();
    out_path.push("out");
    out_path.push("hashed.txt");

    Ok((in_path, out_path))
}
