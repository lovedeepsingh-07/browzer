use uuid::Uuid;
use crate::error::*;
use std::{
    sync::{mpsc, Arc, Mutex},
    thread::{self},
};

type Job = Box<dyn FnOnce() + Send + 'static>;

#[derive(Debug)]
pub struct Worker {
    id: Uuid,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    pub fn new(id: Uuid, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver
                .lock()
                .map_err(ThreadPoolError::from)
                .and_then(|rx| rx.recv().map_err(ThreadPoolError::from));
            match message {
                Ok(job) => {
                    job();
                }
                Err(_) => {
                    println!("Worker {} disconnected, shutting down...", id.to_string());
                    break;
                }
            }
        });

        // return the Worker struct
        return Worker {
            id,
            thread: Some(thread),
        };
    }
}

// The thread pool maintains a set of workers and a channel for sending jobs to them.
#[derive(Debug)]
pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Option<mpsc::Sender<Job>>,
}
impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);
        for _ in 0..size {
            workers.push(Worker::new(Uuid::new_v4(), Arc::clone(&receiver)));
        }

        // return the ThreadPool struct
        return ThreadPool {
            sender: Some(sender),
            workers,
        };
    }

    pub fn execute<F>(&self, f: F) -> Result<(), ThreadPoolError>
    where
        F: FnOnce() + Send + 'static,
    {
        let _ = self
            .sender
            .as_ref()
            .ok_or_else(|| ThreadPoolError::SendError("Sender is not innitialized".to_string()))?
            .send(Box::new(f))
            .map_err(|e| ThreadPoolError::SendError(e.to_string()));
        Ok(())
    }
}

// graceful shutdown
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in &mut self.workers {
            println!("Shuting down worker {}", worker.id.to_string());
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
