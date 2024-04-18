use super::thread_pool_worker::{Job, Worker};
use std::sync::{mpsc, Arc, Mutex};

/// The ThreadPool struct holds a vector of workers and a channel to send jobs to the workers.
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}

impl ThreadPool {
    pub fn new(mut size: usize) -> ThreadPool {
        if size == 0 {
            size = 1;
        }
        let (sender, receiver) = std::sync::mpsc::channel();
        // mutex permite sólo un thread a la vez para acceder a cierta data dada.
        // Arc es una referencia atómicamente contada, se usa en concurrencia.
        let receiver = Arc::new(Mutex::new(receiver));
        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Executes a closure in a thread.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        let Some(sender) = self.sender.as_ref() else {
            println!("Error: self.sender.as_ref()");
            return;
        };
        match sender.send(job) {
            Ok(_) => (),
            Err(_) => println!("Error: sender.send(job)"),
        };
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
        println!("Threadpool dropped");
    }
}

#[cfg(test)]
mod thread_pool_tests {

    use super::*;

    #[test]
    fn test_thread_pool() {
        let pool = ThreadPool::new(4);

        for i in 0..10 {
            pool.execute(move || {
                println!("Thread {} is running", i);
            });
        }
    }

    #[test]
    fn test_new_thread_pool() {
        let pool = ThreadPool::new(4);
        assert!(pool.workers.len() == 4);
    }
}
