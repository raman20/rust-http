use std::{
    sync::{mpsc, Arc, Mutex}, // Import necessary modules from the standard library
    thread,
};

// ThreadPool struct represents a pool of worker threads
pub struct ThreadPool {
    workers: Vec<Worker>,              // A vector to hold the worker threads
    sender: Option<mpsc::Sender<Job>>, // A channel sender to send jobs to the workers
}

// Job type alias represents a closure that can be sent to a worker thread
type Job = Box<dyn FnOnce() + Send + 'static>;

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0); // Ensure that the size is greater than zero

        // Create a new channel for communication between the threads
        let (sender, receiver) = mpsc::channel();

        // Wrap the receiver in an Arc and Mutex for shared ownership and thread safety
        let receiver = Arc::new(Mutex::new(receiver));

        // Create a vector to hold the workers
        let mut workers = Vec::with_capacity(size);

        // Create worker threads and store them in the vector
        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        // Return a new ThreadPool instance
        ThreadPool {
            workers,
            sender: Some(sender),
        }
    }

    /// Execute a closure on a worker thread.
    ///
    /// The closure must be `Send` and `'static` so that it can be safely moved to another thread.
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        // Create a new job from the closure
        let job = Box::new(f);

        // Send the job to a worker thread via the channel
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}

// Implement the Drop trait for ThreadPool to clean up worker threads on drop
impl Drop for ThreadPool {
    fn drop(&mut self) {
        // Drop the sender to close the channel and signal to the workers that there are no more jobs
        drop(self.sender.take());

        // Iterate over the workers and shut them down
        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            // Take the thread from the worker and wait for it to finish
            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}

// Worker struct represents a single worker thread
struct Worker {
    id: usize,                              // The ID of the worker
    thread: Option<thread::JoinHandle<()>>, // The thread handle for the worker
}

impl Worker {
    /// Create a new worker thread.
    ///
    /// The worker will listen for jobs on the receiver and execute them.
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // Spawn a new thread
        let thread = thread::spawn(move || loop {
            // Receive a job from the channel
            let message = receiver.lock().unwrap().recv();

            // Handle the message
            match message {
                Ok(job) => {
                    // Execute the job
                    println!("Worker {id} got a job; executing.");
                    job();
                }
                Err(_) => {
                    // Shut down the worker if the channel is disconnected
                    println!("Worker {id} disconnected; shutting down.");
                    break;
                }
            }
        });

        // Return a new Worker instance
        Worker {
            id,
            thread: Some(thread),
        }
    }
}
