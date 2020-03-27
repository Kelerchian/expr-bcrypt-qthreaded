use crate::utils::get_in_out_paths;
use bcrypt::hash;
use num_cpus;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{self, Write};
use std::io::{BufRead, BufReader};
use std::rc::Rc;
use std::sync::mpsc::channel;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::thread::{self};

struct QThread {
    join_handle: JoinHandle<()>,
    active_arc: Arc<Option<()>>,
    tx: Sender<Option<Box<dyn Fn() -> () + Send>>>,
}

impl QThread {
    fn new() -> QThread {
        let (tx, rx) = channel::<Option<Box<dyn Fn() -> () + Send>>>();
        let join_handle: JoinHandle<()> = thread::spawn(move || loop {
            match rx.recv().unwrap() {
                Some(boxed_fn) => {
                    boxed_fn();
                }
                None => break,
            }
        });

        let active_arc: Arc<Option<()>> = Arc::new(None);
        QThread {
            tx,
            join_handle,
            active_arc,
        }
    }

    fn active_count(&self) -> usize {
        Arc::weak_count(&self.active_arc)
    }

    fn exec(&self, func: Box<dyn Fn() -> () + Send>) {
        let strong = Arc::clone(&self.active_arc);
        self.tx
            .send(Some(Box::new(move || {
                let weak = Arc::downgrade(&strong);
                func();
                drop(weak);
            })))
            .unwrap()
    }

    fn schedule_kill(&self) {
        self.tx.send(None);
    }
}

struct QThreadsRegulator {
    workers: Vec<Rc<QThread>>,
}

impl QThreadsRegulator {
    fn new(worker_num: &usize) -> QThreadsRegulator {
        let mut workers: Vec<Rc<QThread>> = vec![];
        for i in 0..*worker_num {
            workers.push(Rc::new(QThread::new()));
        }
        QThreadsRegulator { workers }
    }

    fn find_available_worker(&self) -> Option<&QThread> {
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

    fn wait_all(&mut self) {
        let workers = &mut self.workers;
        while let Some(worker) = workers.pop() {
            match Rc::try_unwrap(worker) {
                Ok(worker_raw) => worker_raw.join_handle.join().unwrap(),
                Err(_) => eprintln!("join handle failed"),
            };
        }
    }
}

pub fn hash_and_write(pass_u8: &[u8], out_file_mutex: &Arc<Mutex<File>>) -> io::Result<()> {
    let result = hash(&pass_u8, 8);
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
    let out_file_mutex = Arc::new(Mutex::new(
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(out_path.as_os_str())?,
    ));

    let mut worker_regulator = QThreadsRegulator::new(&num_cpus::get());
    let mut lines_iter = BufReader::new(&in_file).lines();
    let mut iteration = 0;
    for line in lines_iter {
        let pass = line?;
        println!("qthreaded count: {}", iteration);
        iteration += 1;
        if pass.len() > 0 {
            loop {
                let pass_clone = pass.clone();
                let out_file_mutex_clone = Arc::clone(&out_file_mutex);
                match worker_regulator.find_available_worker() {
                    Some(worker) => {
                        println!("worker-found");
                        worker.exec(Box::new(move || {
                            let pass_u8 = pass_clone.as_bytes();
                            match hash_and_write(&pass_u8, &out_file_mutex_clone) {
                                Ok(_) => (),
                                Err(_) => (),
                            }
                        }));
                        break;
                    },
                    None => {
                        println!("no-worker-found");
                        std::thread::sleep(std::time::Duration::from_millis(5));
                    },
                }
            }
        }
    }
    worker_regulator.schedule_kill();
    worker_regulator.wait_all();

    Ok(())
}
