use std::thread;
use crossbeam_channel::{bounded, Sender};

pub struct ThreadPool {
    threads: Vec<thread::JoinHandle<()>>,
    task_sender: Sender<Box<dyn FnOnce() + Send>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        let mut threads: Vec<thread::JoinHandle<()>> = Vec::with_capacity(size);

        let (task_sender, task_receiver) = bounded::<Box<dyn FnOnce() + Send>>(size);

        for _ in 0..size {
            let task_receiver = task_receiver.clone();

            let handle = thread::spawn(move || {
                for task in task_receiver {
                    task();
                }
            });

            threads.push(handle);
        }

        ThreadPool {
            task_sender,
            threads,
        }
    }

    pub fn spawn<T: FnOnce() + Send + 'static>(&self, task: T) {
        let task = Box::new(task);

        self.task_sender
            .send(task)
            .unwrap();
    }

    pub fn join(self) {
        drop(self.task_sender);

        self.threads
            .into_iter()
            .for_each(|thread| thread.join().unwrap());
    }
}

#[cfg(test)]
mod tests {
    mod thread_pool {
        use std::sync::{Arc, Mutex};
        use crate::thread_pool::ThreadPool;

        #[test]
        fn test_thread_pool() {
            let pool = ThreadPool::new(1);
            let result = Arc::new(Mutex::new(false));

            let result_clone = result.clone();
            pool.spawn(move || {
                *result_clone.lock().unwrap() = true;
            });
            pool.join();

            assert_eq!(
                *result.lock().unwrap(),
                true
            );
        }
    }
}
