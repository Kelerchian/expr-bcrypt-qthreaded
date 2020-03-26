use bcrypt::hash;
use std::fs::{OpenOptions};
use std::io::{self, BufRead, BufReader, Write};
use crate::utils::get_in_out_paths;

pub fn sequential() -> io::Result<()> {

    let (in_path, out_path) = get_in_out_paths()?;
    let in_file = OpenOptions::new().read(true).open(in_path.as_os_str())?;
    let mut out_file = OpenOptions::new().read(true).write(true).create(true).open(out_path.as_os_str())?;
    let lines = BufReader::new(in_file).lines();

    for line in lines {
        let pass = line?;
        if pass.len() > 0 {
            let result = hash(pass, 4);
            if let Ok(hashed) = result {
                out_file.write(hashed.as_bytes())?;
            } else {
                out_file.write("hash-failed".as_bytes())?;
            }
            out_file.write("\n".as_bytes())?;
        };
    }

    Ok(())
}
