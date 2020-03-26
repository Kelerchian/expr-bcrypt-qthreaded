mod bcrypt_sequential;
mod bcrypt_qthreaded;
mod utils;

use std::io;
use std::time::Instant;
use bcrypt_sequential::sequential;
use bcrypt_qthreaded::qthreaded;

fn main() -> io::Result<()> {

    let time = Instant::now();
    sequential()?;
    println!("Sequential: time elapsed: {} secs", time.elapsed().as_secs());

    let time = Instant::now();
    qthreaded()?;
    println!("Qthreaded: time elapsed: {} secs", time.elapsed().as_secs());

    Ok(())
}
