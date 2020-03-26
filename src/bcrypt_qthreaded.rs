use std::fs::File;
use std::rc::Rc;
use std::sync::mpsc::{Sender,Receiver};
use std::thread::JoinHandle;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use crate::utils::get_in_out_paths;
use std::fs::{OpenOptions};
use std::io::{self, Write};
use std::thread::{self};
use num_cpus;
use std::io::{BufRead, BufReader};
use bcrypt::{hash};

struct Worker {
    join_handle: JoinHandle<()>,
    tx: Sender<Option<Box<dyn Fn() -> () + Send>>>,
    active_arc: Arc<Option<()>>
}

impl Worker {
    fn new() -> Worker {
        let (tx, rx) = channel::<Option<Box<dyn Fn() -> () + Send>>>();
        let join_handle: JoinHandle<()> = thread::spawn(move || {
            loop {
                match rx.recv().unwrap() {
                    Some(boxed_fn) => {
                        boxed_fn();
                    }
                    None => break,
                }
            }
        });

        let active_arc: Arc<Option<()>> = Arc::new(None);
        Worker { tx, join_handle, active_arc }
    }

    fn active_count(&self) -> usize {
        Arc::weak_count(&self.active_arc)
    }

    fn exec(&self, func: Box<dyn Fn() -> () + Send>) {
        let strong = Arc::clone(&self.active_arc);
        self.tx.send(Some(Box::new(move || {
            let weak = Arc::downgrade(&strong);
            func();
            drop(weak);
        }))).unwrap()
    }

    fn schedule_kill(&self) {
        self.tx.send(None);
    }
}

struct WorkerRegulator {
    workers: Vec<Rc<Worker>>
}

impl WorkerRegulator {
    fn new(num: &usize) -> WorkerRegulator {
        let mut workers: Vec<Rc<Worker>> = vec![];
        for _ in 1..*num {
            workers.push(Rc::new(Worker::new()));
        };
        WorkerRegulator { workers }
    }

    fn find_available_worker(&self) -> Option<&Worker> {
        for worker in &self.workers {
            if worker.active_count() == 0 {
                return Some(worker);
            }
        }
        None
    }

    fn schedule_kill(&self) {
        for worker in &self.workers {
            worker.schedule_kill();
        }
    }

    fn wait_all(&mut self){
        let workers = &mut self.workers;
        while let Some(worker) = workers.pop() {
            if let Ok(worker) = Rc::downgrade(&worker) {
                worker.join_handle.
            }
        }
    }
}

pub fn hash_and_write(pass_u8: &[u8], out_file_mutex: &Arc<Mutex<File>>) -> io::Result<()> {
    let result = hash(&pass_u8, 4);
    let mut out_file = out_file_mutex.lock().unwrap();
    if let Ok(hashed) = result {
        out_file.write(hashed.as_bytes())?;
    } else {
        out_file.write("hash-failed".as_bytes())?;
    }
    out_file.write("\n".as_bytes())?;

    Ok(())
}

pub fn qthreaded() -> io::Result<()> {

    let (in_path, out_path) = get_in_out_paths()?;
    let in_file = OpenOptions::new().read(true).open(in_path.as_os_str())?;
    let out_file_mutex = Arc::new(Mutex::new(OpenOptions::new().read(true).write(true).create(true).open(out_path.as_os_str())?));

    let worker_regulator = WorkerRegulator::new(&num_cpus::get());

    loop {
        let mut line: Vec<u8> = vec![];
        let mut reader = BufReader::new(&in_file);
        match reader.read_until(b'\n', &mut line) {
            Ok(0) => break,
            Ok(_) => if line.len() > 0 {
                loop {
                    let pass_u8 = line.clone();
                    let out_file_mutex_clone = Arc::clone(&out_file_mutex);
                    match worker_regulator.find_available_worker() {
                        Some(worker) => worker.exec(Box::new(move || match hash_and_write(&pass_u8, &out_file_mutex_clone) {
                            Ok(_) => (),
                            Err(_) => ()
                        })),
                        None => ()
                    }
                }
            },
            Err(x) => drop(x),
        };
    }
    worker_regulator.schedule_kill();

    Ok(())
}
