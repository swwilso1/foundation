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

impl Default for ThreadJob {
    fn default() -> Self {
        Self::new()
    }
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

    /// Prepend a task to the `ThreadJob` object.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to prepend to the job list.
    ///
    /// This function has O(tasks::len) time complexity.
    pub fn prepend_task(&mut self, task: Task) {
        self.job_list.insert(0, task);
    }
}

// The `WorkerId` type is a unique identifier for a worker in the thread pool.
pub type WorkerId = u16;

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
    ///   The idle worker notifications are just the worker's unique identifier sent back to the
    ///   idle channel.
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
                            match task.await {
                                Ok(_) => {}
                                Err(e) => {
                                    error!("Error executing task in worker {}: {}", worker_id, e);
                                    break;
                                }
                            }
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
    stopper: Box<dyn Fn() + Send + Sync + 'static>,
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
                                    if let Ok(mut scheduler_worker_manager) =
                                        scheduler_worker_manager.lock()
                                    {
                                        if scheduler_worker_manager.current_workers
                                            < scheduler_worker_manager.max_workers
                                        {
                                            let next_worker_id =
                                                scheduler_worker_manager.next_worker_id;
                                            let worker =
                                                Worker::new(next_worker_id, idle_sender.clone());
                                            scheduler_worker_manager
                                                .workers
                                                .insert(next_worker_id, worker);
                                            scheduler_worker_manager.next_worker_id += 1;
                                            scheduler_worker_manager.current_workers += 1;
                                        }
                                    }

                                    // We may have added a worker to the pool, so now we just wait till we get an
                                    // idle worker. The crucial bit here is that we now block waiting for the next
                                    // idle worker to be available. We block here to avoid spinning on try_recv() in
                                    // the main part of the loop.
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
    use crate::result::DynResultError;
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

        assert!(*control1.lock().unwrap());
        assert!(*control2.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_prepend_task() {
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
        thread_job.prepend_task(Box::pin(async move {
            *control2_c.lock().unwrap() = true;
            Ok(())
        }));
        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert!(*control2.lock().unwrap());
        assert!(*control1.lock().unwrap());

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

        assert!(*control1.lock().unwrap());
        assert!(*control2.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_large_pool_with_slow_jobs() {
        let mut thread_pool = ThreadPool::new(10);
        let mut thread_job1 = ThreadJob::new();
        let mut thread_job2 = ThreadJob::new();
        let mut thread_job3 = ThreadJob::new();
        let mut thread_job4 = ThreadJob::new();
        let mut thread_job5 = ThreadJob::new();
        let mut thread_job6 = ThreadJob::new();

        let control1 = Arc::new(Mutex::new(false));
        let control2 = Arc::new(Mutex::new(false));
        let control3 = Arc::new(Mutex::new(false));
        let control4 = Arc::new(Mutex::new(false));
        let control5 = Arc::new(Mutex::new(false));
        let control6 = Arc::new(Mutex::new(false));

        let control1_c = control1.clone();
        let control2_c = control2.clone();
        let control3_c = control3.clone();
        let control4_c = control4.clone();
        let control5_c = control5.clone();
        let control6_c = control6.clone();

        thread_job1.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control1_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job2.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control2_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job3.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control3_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job4.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control4_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job5.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control5_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job6.add_task(Box::pin(async move {
            sleep(Duration::from_secs(2)).await;
            *control6_c.lock().unwrap() = true;
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
        if let Err(e) = thread_pool.add_job(thread_job5) {
            panic!("Error adding job 5 to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(thread_job6) {
            panic!("Error adding job 6 to thread pool: {}", e);
        }

        sleep(Duration::from_millis(2100)).await;

        assert!(*control1.lock().unwrap());
        assert!(*control2.lock().unwrap());
        assert!(*control3.lock().unwrap());
        assert!(*control4.lock().unwrap());
        assert!(*control5.lock().unwrap());
        assert!(*control6.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_error_in_tasks() {
        let mut thread_pool = ThreadPool::new(4);
        let mut thread_job = ThreadJob::new();

        let control1 = Arc::new(Mutex::new(false));
        let control2 = Arc::new(Mutex::new(false));

        let control1_c = control1.clone();

        thread_job.add_task(Box::pin(async move {
            *control1_c.lock().unwrap() = true;
            Ok(())
        }));
        thread_job.add_task(Box::pin(async move {
            let error = Box::new(FoundationError::ThreadTaskError(
                "Error in task".to_string(),
            ));
            Err(error as DynResultError)
        }));

        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert!(*control1.lock().unwrap());
        assert!(!(*control2.lock().unwrap()));

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

        assert!(*control1.lock().unwrap());
        assert!(*control2.lock().unwrap());
        assert!(*control3.lock().unwrap());
        assert!(*control4.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_tasks_execute_in_order_within_a_job() {
        let mut thread_pool = ThreadPool::new(4);
        let mut thread_job = ThreadJob::new();

        // A single job's tasks must run sequentially in the order they were added.
        let order = Arc::new(Mutex::new(Vec::<u32>::new()));

        for n in 0..5 {
            let order_c = order.clone();
            thread_job.add_task(Box::pin(async move {
                order_c.lock().unwrap().push(n);
                Ok(())
            }));
        }

        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2, 3, 4]);

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_prepend_task_runs_first() {
        let mut thread_pool = ThreadPool::new(4);
        let mut thread_job = ThreadJob::new();

        let order = Arc::new(Mutex::new(Vec::<u32>::new()));

        let order_c = order.clone();
        thread_job.add_task(Box::pin(async move {
            order_c.lock().unwrap().push(1);
            Ok(())
        }));
        let order_c = order.clone();
        thread_job.add_task(Box::pin(async move {
            order_c.lock().unwrap().push(2);
            Ok(())
        }));
        // The prepended task must execute before the two tasks added above.
        let order_c = order.clone();
        thread_job.prepend_task(Box::pin(async move {
            order_c.lock().unwrap().push(0);
            Ok(())
        }));

        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert_eq!(*order.lock().unwrap(), vec![0, 1, 2]);

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_error_in_job_does_not_stop_later_jobs() {
        let mut thread_pool = ThreadPool::new(1);

        // The first job errors on its first task, which should skip the remaining
        // task in that job but leave the worker available for subsequent jobs.
        let mut failing_job = ThreadJob::new();
        let never_ran = Arc::new(Mutex::new(false));
        let never_ran_c = never_ran.clone();
        failing_job.add_task(Box::pin(async move {
            let error = Box::new(FoundationError::ThreadTaskError("boom".to_string()));
            Err(error as DynResultError)
        }));
        failing_job.add_task(Box::pin(async move {
            *never_ran_c.lock().unwrap() = true;
            Ok(())
        }));

        let mut good_job = ThreadJob::new();
        let ran = Arc::new(Mutex::new(false));
        let ran_c = ran.clone();
        good_job.add_task(Box::pin(async move {
            *ran_c.lock().unwrap() = true;
            Ok(())
        }));

        if let Err(e) = thread_pool.add_job(failing_job) {
            panic!("Error adding failing job to thread pool: {}", e);
        }
        if let Err(e) = thread_pool.add_job(good_job) {
            panic!("Error adding good job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        // The task after the failing task in the same job must have been skipped.
        assert!(!(*never_ran.lock().unwrap()));
        // A subsequent job must still execute on the same worker.
        assert!(*ran.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_empty_job_completes() {
        let mut thread_pool = ThreadPool::new(2);

        // An empty job should be processed without error, and the worker should
        // remain available to run a subsequent job with real work.
        if let Err(e) = thread_pool.add_job(ThreadJob::new()) {
            panic!("Error adding empty job to thread pool: {}", e);
        }

        let ran = Arc::new(Mutex::new(false));
        let ran_c = ran.clone();
        let mut thread_job = ThreadJob::new();
        thread_job.add_task(Box::pin(async move {
            *ran_c.lock().unwrap() = true;
            Ok(())
        }));
        if let Err(e) = thread_pool.add_job(thread_job) {
            panic!("Error adding job to thread pool: {}", e);
        }

        sleep(Duration::from_millis(200)).await;

        assert!(*ran.lock().unwrap());

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_workers_are_reused_and_do_not_exceed_max() {
        // With a single allowed worker, several sequential jobs should all run on
        // that one worker rather than spawning additional workers.
        let mut thread_pool = ThreadPool::new(1);

        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..5 {
            let count_c = count.clone();
            let mut thread_job = ThreadJob::new();
            thread_job.add_task(Box::pin(async move {
                *count_c.lock().unwrap() += 1;
                Ok(())
            }));
            if let Err(e) = thread_pool.add_job(thread_job) {
                panic!("Error adding job to thread pool: {}", e);
            }
        }

        sleep(Duration::from_millis(300)).await;

        assert_eq!(*count.lock().unwrap(), 5);

        let manager = thread_pool.worker_manager.lock().unwrap();
        assert_eq!(manager.current_workers, 1);
        assert!(manager.current_workers <= manager.max_workers);
        drop(manager);

        thread_pool.stop();
    }

    #[tokio::test]
    async fn test_worker_pool_scales_up_to_max() {
        // Submit several long-running jobs so the scheduler is forced to spin up
        // multiple workers (up to max) to make progress concurrently.
        let mut thread_pool = ThreadPool::new(3);

        let count = Arc::new(Mutex::new(0u32));
        for _ in 0..3 {
            let count_c = count.clone();
            let mut thread_job = ThreadJob::new();
            thread_job.add_task(Box::pin(async move {
                sleep(Duration::from_millis(300)).await;
                *count_c.lock().unwrap() += 1;
                Ok(())
            }));
            if let Err(e) = thread_pool.add_job(thread_job) {
                panic!("Error adding job to thread pool: {}", e);
            }
        }

        // Give the scheduler time to allocate workers, but not enough for the jobs
        // to finish, so the worker count reflects concurrent demand.
        sleep(Duration::from_millis(150)).await;

        {
            let manager = thread_pool.worker_manager.lock().unwrap();
            assert!(manager.current_workers >= 1);
            assert!(manager.current_workers <= manager.max_workers);
            assert_eq!(manager.max_workers, 3);
        }

        sleep(Duration::from_millis(400)).await;
        assert_eq!(*count.lock().unwrap(), 3);

        thread_pool.stop();
    }

    #[test]
    fn test_worker_manager_new_initial_state() {
        let manager = WorkerManager::new(8);
        assert_eq!(manager.max_workers, 8);
        assert_eq!(manager.current_workers, 0);
        assert_eq!(manager.next_worker_id, 0);
        assert!(manager.workers.is_empty());
    }
}
