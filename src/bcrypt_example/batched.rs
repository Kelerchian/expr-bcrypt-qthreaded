use bcrypt::hash;
use std::fs::{OpenOptions, File};
use std::io::{self, BufRead, BufReader, Write, Lines};
use std::thread;
use num_cpus;
use crate::utils::get_in_out_paths;

pub fn read_lines_batched(lines: &mut Lines<BufReader<File>>, max: &usize) -> Option<Vec<String>> {
    let mut batched_lines = vec![];
    while batched_lines.len() < *max {
        match lines.next() {
            Some(line_res) => match line_res {
                Ok(line) => batched_lines.push(line),
                Err(_) => continue,
            },
            None => break,
        }
    }
    return match batched_lines.len() > 0 {
        true => Some(batched_lines),
        false => None
    };
}

pub fn batched() -> io::Result<()> {

    let (in_path, out_path) = get_in_out_paths()?;
    let in_file = OpenOptions::new().read(true).open(in_path.as_os_str())?;
    let mut out_file = OpenOptions::new().read(true).write(true).create(true).open(out_path.as_os_str())?;
    let mut lines = BufReader::new(in_file).lines();
    let threads_num = num_cpus::get();

    let mut iteration = 0;

    while let Some(batched_lines) = read_lines_batched(&mut lines, &threads_num) {
        println!("batched iteration: {}", &iteration);
        iteration += 1;
        batched_lines
            .into_iter()
            .map(|line| thread::spawn(move || {
                match hash(line.as_bytes(), 8) {
                    Ok(hashed) => hashed,
                    Err(_) => String::from("hash-failed")
                }
            }))
            .map(|join_handle| join_handle.join())
            .for_each(|hashed| {
                let res = match hashed {
                    Ok(res) => String::from(res),
                    Err(e) => String::from("hash-failed"),
                };
                out_file.write(res.as_bytes());
            });
    }

//    for line in lines {
//        let pass = line?;
//        println!("sequential iteration: {}", iteration);
//        iteration += 1;
//        if pass.len() > 0 {
//            let result = hash(pass, 8);
//            if let Ok(hashed) = result {
//                out_file.write(hashed.as_bytes())?;
//            } else {
//                out_file.write("hash-failed".as_bytes())?;
//            }
//            out_file.write("\n".as_bytes())?;
//        };
//    }

    Ok(())
}
