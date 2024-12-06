use std::boxed::Box;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::log_info;
type Job = Box<dyn FnOnce() + Send + 'static>;
pub(crate) struct Worker {
    pub id: usize,
    pub worker: Option<thread::JoinHandle<()>>,
}
impl Worker {
    fn new(id: usize, reciever: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thrd = thread::spawn(move || loop {
            let lock = reciever.lock().unwrap();
            let msg = lock.recv().unwrap();
            drop(lock);
            match msg {
                Message::NewJob(job) => {
                    // dbg!(format!("Worker {} got a job to execute.",id));
                    job();
                }
                Message::Terminate => {
                    break;
                }
            }
        });
        Worker {
            id,
            worker: Some(thrd),
        }
    }
}
pub enum Message {
    NewJob(Job),
    Terminate,
}
pub(crate) struct ThreadPool {
    pub workers: Vec<Worker>,
    pub sender: mpsc::Sender<Message>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);
        let mut workers: Vec<Worker> = Vec::with_capacity(size);
        let (sender, reciever) = mpsc::channel();
        let reciever = Arc::new(Mutex::new(reciever));
        for id in 0..size {
            workers.push(Worker::new(id, reciever.clone()));
        }
        ThreadPool { workers, sender }
    }
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let job = Message::NewJob(job);
        self.sender.send(job).unwrap();
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        let s = format!("Terminatig all {} workers.", self.workers.len());
        log_info!(s);
        for _ in &self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }
        for worker in &mut self.workers {
            if let Some(thrd) = worker.worker.take() {
                thrd.join().unwrap();
            }
        }
        log_info!("All workers are succesfully terminated.");
    }
}
