use std::env;
use std::io;
use std::io::{Write};
use std::path::PathBuf;
use std::fs::OpenOptions;
use passwords::PasswordGenerator;

fn get_in_out_dir() -> io::Result<(PathBuf,PathBuf)> {
    let mut dir = env::current_dir()?;
    dir.pop();
    dir.pop();
    dir.push("data");
    dir.push("in");

    let mut in_dir = dir.clone();
    in_dir.push("password.txt");

    let mut in_small_dir = dir.clone();
    in_small_dir.push("password-small.txt");

    Ok((in_dir, in_small_dir))
}

fn main() -> io::Result<()> {
    let (in_dir, in_small_dir) = get_in_out_dir()?;
    let pass_gen = PasswordGenerator::new();

    let mut in_file = OpenOptions::new().read(true).write(true).create(true).open(in_dir.as_os_str())?;
    let mut in_small_file = OpenOptions::new().read(true).write(true).create(true).open(in_small_dir.as_os_str())?;

    for n in 0..2000 {
        let pass = pass_gen.generate_one().unwrap();
        if n < 2000 {
            in_small_file.write(pass.as_bytes())?;
            in_small_file.write("\n".as_bytes())?;
        }
        in_file.write(pass.as_bytes())?;
        in_file.write("\n".as_bytes())?;
    }

    Ok(())
}
