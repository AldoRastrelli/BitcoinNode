use std::sync::Arc;
use std::sync::Mutex;
use std::thread::{self, JoinHandle};

pub struct Worker {
    pub id: usize,
    pub thread: Option<JoinHandle<()>>,
}

pub type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    pub fn new(id: usize, receiver: Arc<Mutex<std::sync::mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = if let Ok(receiver) = receiver.lock() {
                match receiver.recv() {
                    Ok(message) => message,
                    Err(_) => {
                        println!("Worker {id} disconnected (recv); shutting down.");
                        break;
                    }
                }
            } else {
                println!("Worker {id} disconnected (lock); shutting down.");
                break;
            };
            println!("Worker {id} got a job; executing.");
            job();
        });

        Worker {
            id,
            thread: Some(thread),
        }
    }
}

#[cfg(test)]
mod thread_pool_worker_tests {
    use super::*;

    #[test]
    fn test_worker_is_some() {
        let (_sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let worker = Worker::new(0, Arc::clone(&receiver));
        assert!(worker.thread.is_some());
    }

    #[test]
    fn test_worker_id() {
        let (_sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let worker = Worker::new(0, Arc::clone(&receiver));
        assert_eq!(worker.id, 0);
    }

    #[test]
    fn test_worker_receiver() {
        let (_sender, receiver) = std::sync::mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));
        let worker = Worker::new(0, Arc::clone(&receiver));
        assert!(worker.thread.is_some());
    }
}
