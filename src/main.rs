mod bcrypt_example;
mod utils;

use std::io;
use std::time::Instant;
use bcrypt_example::{
    sequential::sequential,
    qthreaded::qthreaded,
    batched::batched,
};

fn main() -> io::Result<()> {

    let time = Instant::now();
    qthreaded()?;
    let qthreaded_time_elapsed = time.elapsed().as_millis();

    let time = Instant::now();
    batched()?;
    let batched_time_elapsed = time.elapsed().as_millis();

    let time = Instant::now();
    sequential()?;
    let sequential_time_elapsed = time.elapsed().as_millis();

    println!("Qthreaded  - time elapsed: {} secs", qthreaded_time_elapsed);
    println!("Batched    - time elapsed: {} secs", batched_time_elapsed);
    println!("Sequential - time elapsed: {} secs", sequential_time_elapsed);

    Ok(())
}
