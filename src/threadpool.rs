//! The `threadpool` module provides an asynchronous thread pool for running tasks.

use crate::error::FoundationError;
use crate::result::DynResult;
use log::{debug, error};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use tokio::{
    spawn,
    sync::mpsc::{error::TryRecvError, unbounded_channel, UnboundedSender},
    task::JoinHandle,
};

/// The `Task` type is the basic closure type that encapsulates the work to be done in the thread pool.
pub type Task = Pin<Box<dyn Future<Output = DynResult<()>> + Send + Sync + 'static>>;

/// The `ThreadJob` type is a collection of `Task` types that are to be executed in the thread pool.
/// Tasks in the `ThreadJob` are executed in the order they are added to the job list by a single
/// thread in the thread pool. Use multiple tasks if you need to have jobs executed in the pool
/// that depend on the order of execution.
pub struct ThreadJob {
    // The list of tasks to be executed in the thread pool thread.
    job_list: Vec<Task>,
}

impl ThreadJob {
    /// Create a new `ThreadJob` object.
    pub fn new() -> ThreadJob {
        ThreadJob {
            job_list: Vec::new(),
        }
    }

    /// Add a task to the `ThreadJob` object.
    pub fn add_task(&mut self, task: Task) {
        self.job_list.push(task);
    }
}

// The `WorkerId` type is a unique identifier for a worker in the thread pool.
type WorkerId = u16;

// The `Worker` type is a single worker in the thread pool. It is responsible for executing tasks
// in a `ThreadJob`.
struct Worker {
    // The sender channel for sending jobs to the worker thread.
    job_sender: UnboundedSender<ThreadJob>,

    // The stopper function for stopping the worker thread.
    stopper: Box<dyn Fn() -> DynResult<()> + Send + Sync + 'static>,
}

impl Worker {
    /// Create a new `Worker` object.
    ///
    /// # Arguments
    ///
    /// * `id` - The unique identifier for the worker.
    /// * `idle_sender` - The sender channel for sending idle worker notifications.
    /// The idle worker notifications are just the worker's unique identifier sent back to the
    /// idle channel.
    ///
    /// # Returns
    ///
    /// A new `Worker` object.
    pub fn new(id: WorkerId, idle_sender: UnboundedSender<WorkerId>) -> Worker {
        let (job_sender, mut job_receiver) = unbounded_channel::<ThreadJob>();

        let worker_id = id;
        let worker_idle_sender = idle_sender.clone();

        let thread: JoinHandle<DynResult<()>> = spawn(async move {
            debug!("Starting thread pool worker {}", worker_id);
            loop {
                // Wait for the next job.
                let job = job_receiver.recv().await;
                if let Some(mut job) = job {
                    loop {
                        // Execute all the tasks in the job.
                        for task in job.job_list {
                            task.await?;
                        }

                        // Now check to see if we have another job in the channel.
                        match job_receiver.try_recv() {
                            Ok(new_job) => {
                                // We have a job, just replace the current job with the new one and
                                // try to execute those tasks after we loop back around.
                                job = new_job
                            }
                            Err(e) => {
                                match e {
                                    TryRecvError::Empty => {
                                        // We do not have any more jobs, so we are now idle. Send
                                        // the idle channel our id so that the scheduler can schedule
                                        // more work for us when the scheduler has more jobs.
                                        worker_idle_sender.send(worker_id)?;
                                        break;
                                    }
                                    TryRecvError::Disconnected => {
                                        debug!(
                                            "Worker {} received a disconnect from the job sender.",
                                            id
                                        );
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        });

        // TODO: Should we return an error, instead of logging an error?
        if let Err(e) = idle_sender.send(id) {
            error!(
                "Unable to send initial idle message for worker {} to scheduler: {}",
                id, e
            );
        }

        Worker {
            job_sender,
            // We use a closure to stop the thread worker because storing the JoinHandle in the
            // Worker structure is problematic when we want to call the stopper function.
            stopper: Box::new(move || {
                thread.abort();
                Ok(())
            }),
        }
    }

    /// Add a job to the worker.
    ///
    /// # Arguments
    ///
    /// * `job` - The job to add to the worker.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn add_job(&mut self, job: ThreadJob) -> Result<(), FoundationError> {
        match self.job_sender.send(job) {
            Ok(_) => Ok(()),
            Err(e) => Err(FoundationError::TokioMpscSend(e.to_string())),
        }
    }

    /// Stop the worker.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn stop(&mut self) -> DynResult<()> {
        (self.stopper)()
    }
}

struct WorkerManager {
    // The map of workers in the thread pool.
    pub workers: HashMap<WorkerId, Worker>,

    // The next worker id to use when creating a new worker.
    pub next_worker_id: WorkerId,

    // The current number of workers in the thread pool.
    pub current_workers: WorkerId,

    // The maximum number of workers in the thread pool.
    pub max_workers: WorkerId,
}

impl WorkerManager {
    pub fn new(max_workers: WorkerId) -> WorkerManager {
        WorkerManager {
            workers: HashMap::new(),
            next_worker_id: 0,
            current_workers: 0,
            max_workers,
        }
    }
}

// The `ThreadPool` type is the main thread pool object. It is responsible for managing the
// scheduler thread and the worker threads.
pub struct ThreadPool {
    // The sender channel for sending jobs to the scheduler thread.
    job_sender: UnboundedSender<ThreadJob>,

    // The worker manager.
    worker_manager: Arc<Mutex<WorkerManager>>,

    // The stopper function for stopping the scheduler thread.
    stopper: Box<dyn Fn() -> () + Send + Sync + 'static>,
}

impl ThreadPool {
    /// Create a new `ThreadPool` object.
    ///
    /// # Arguments
    ///
    /// * `idle_receiver` - The receiver channel for receiving idle worker notifications.
    ///
    /// # Returns
    ///
    /// A new `ThreadPool` object.
    pub fn new(max_workers: WorkerId) -> ThreadPool {
        // Create the channe for sending ThreadJobs to the scheduler thread.
        let (job_sender, mut job_receiver) = unbounded_channel::<ThreadJob>();

        // Create the map of workers in the thread pool.
        // The map is a shared resource between the scheduler and the `ThreadPool`.
        let worker_manager: Arc<Mutex<WorkerManager>> =
            Arc::new(Mutex::new(WorkerManager::new(max_workers)));

        // Clone the manager, so we can use it in the scheduler thread.
        let scheduler_worker_manager = worker_manager.clone();

        // Create the channel for sending idle worker notifications.
        let (idle_sender, mut idle_receiver) = unbounded_channel::<WorkerId>();

        let scheduler: JoinHandle<Result<(), FoundationError>> = spawn(async move {
            debug!("Starting thread pool scheduler");
            loop {
                // Wait for the next job.
                let job = job_receiver.recv().await;
                if let Some(job) = job {
                    // Try to get the next idle worker.  We try here and do not just wait in
                    // the recv() call because we may be able to add a new worker to the pool
                    // if we have not reached the maximum number of workers.
                    match idle_receiver.try_recv() {
                        Ok(idle_worker) => {
                            // Get the worker object, so we can add the job to the worker thread
                            // channel.
                            if let Some(worker) = scheduler_worker_manager
                                .lock()
                                .unwrap()
                                .workers
                                .get_mut(&idle_worker)
                            {
                                worker.add_job(job)?;
                            } else {
                                // TODO: Do we want to drop the job?
                                error!(
                                    "ThreadPool could not find worker {}, dropping job.",
                                    idle_worker
                                );
                            }
                        }
                        Err(e) => {
                            match e {
                                TryRecvError::Empty => {
                                    // There should be a better way to lock the manager and access the contents without
                                    // using the lock method repeatedly to access the contents.
                                    let current_workers =
                                        scheduler_worker_manager.lock().unwrap().current_workers;
                                    let max_workers =
                                        scheduler_worker_manager.lock().unwrap().max_workers;
                                    if current_workers < max_workers {
                                        let next_worker_id =
                                            scheduler_worker_manager.lock().unwrap().next_worker_id;
                                        let worker =
                                            Worker::new(next_worker_id, idle_sender.clone());
                                        scheduler_worker_manager
                                            .lock()
                                            .unwrap()
                                            .workers
                                            .insert(next_worker_id, worker);
                                        scheduler_worker_manager.lock().unwrap().next_worker_id +=
                                            1;
                                        scheduler_worker_manager.lock().unwrap().current_workers +=
                                            1;
                                    }

                                    // We may have added a worker to the pool, so now we just wait till we get an
                                    // idle worker.
                                    let idle_worker = idle_receiver.recv().await;
                                    if let Some(idle_worker) = idle_worker {
                                        // Get the worker object, so we can add the job to the worker thread
                                        // channel.
                                        if let Some(worker) = scheduler_worker_manager
                                            .lock()
                                            .unwrap()
                                            .workers
                                            .get_mut(&idle_worker)
                                        {
                                            worker.add_job(job)?;
                                        } else {
                                            error!("ThreadPool could not find worker {}, dropping job.", idle_worker);
                                        }
                                    }
                                }
                                TryRecvError::Disconnected => {
                                    debug!("ThreadPool received a disconnect from the idle worker sender.");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }
            }
        });

        ThreadPool {
            job_sender,
            worker_manager,
            stopper: Box::new(move || {
                scheduler.abort();
            }),
        }
    }

    /// Add a job to the pool.
    ///
    /// # Arguments
    ///
    /// * `job` - The job to add to the pool.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    pub fn add_job(&mut self, job: ThreadJob) -> Result<(), FoundationError> {
        match self.job_sender.send(job) {
            Ok(_) => Ok(()),
            Err(e) => Err(FoundationError::TokioMpscSend(e.to_string())),
        }
    }

    /// Stop the pool.
    pub fn stop(&mut self) {
        (self.stopper)();
        for worker in self.worker_manager.lock().unwrap().workers.values_mut() {
            if let Err(e) = worker.stop() {
                error!("Error stopping worker: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_multiple_tasks() {
        let mut thread_pool = ThreadPool::new(4);
        let mut thread_job = ThreadJob::new();

        let control1 = Arc::new(Mutex::new(false));
        let control2 = Arc::new(Mutex::new(false));

        let control1_c = control1.clone();
        let control2_c = control2.clone();

        thread_job.add_task(Box::pin(async move {
            *control1_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job.add_task(Box::pin(async move {
            *control2_c.lock().unwrap() = true;
            Ok(())
        }));
        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert_eq!(*control1.lock().unwrap(), true);
        assert_eq!(*control2.lock().unwrap(), true);

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_multiple_jobs() {
        let mut thread_pool = ThreadPool::new(4);
        let mut thread_job1 = ThreadJob::new();
        let mut thread_job2 = ThreadJob::new();

        let control1 = Arc::new(Mutex::new(false));
        let control2 = Arc::new(Mutex::new(false));

        let control1_c = control1.clone();
        let control2_c = control2.clone();

        thread_job1.add_task(Box::pin(async move {
            *control1_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job2.add_task(Box::pin(async move {
            *control2_c.lock().unwrap() = true;
            Ok(())
        }));
        if let Err(e) = thread_pool.add_job(thread_job1) {
            panic!("Error adding job to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(thread_job2) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert_eq!(*control1.lock().unwrap(), true);
        assert_eq!(*control2.lock().unwrap(), true);

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_more_jobs_than_threads() {
        let mut thread_pool = ThreadPool::new(2);
        let mut thread_job1 = ThreadJob::new();
        let mut thread_job2 = ThreadJob::new();
        let mut thread_job3 = ThreadJob::new();
        let mut thread_job4 = ThreadJob::new();

        let control1 = Arc::new(Mutex::new(false));
        let control2 = Arc::new(Mutex::new(false));
        let control3 = Arc::new(Mutex::new(false));
        let control4 = Arc::new(Mutex::new(false));

        let control1_c = control1.clone();
        let control2_c = control2.clone();
        let control3_c = control3.clone();
        let control4_c = control4.clone();

        thread_job1.add_task(Box::pin(async move {
            *control1_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job2.add_task(Box::pin(async move {
            *control2_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job3.add_task(Box::pin(async move {
            *control3_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job4.add_task(Box::pin(async move {
            *control4_c.lock().unwrap() = true;
            Ok(())
        }));
        if let Err(e) = thread_pool.add_job(thread_job1) {
            panic!("Error adding job 1 to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(thread_job2) {
            panic!("Error adding job 2 to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(thread_job3) {
            panic!("Error adding job 3 to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(thread_job4) {
            panic!("Error adding job 4 to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert_eq!(*control1.lock().unwrap(), true);
        assert_eq!(*control2.lock().unwrap(), true);
        assert_eq!(*control3.lock().unwrap(), true);
        assert_eq!(*control4.lock().unwrap(), true);

        thread_pool.stop();
    }
}
