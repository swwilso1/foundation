//! The `progress_meter` module provides a progress meter that can be used to track the progress
//! of a task.

use num::{FromPrimitive, Integer};
use std::pin::Pin;

/// A callback function that takes a single argument and returns nothing.
pub type Callback<T> = Pin<Box<dyn Fn(T) -> () + Send + Sync + 'static>>;

/// The `ProgressMeterNotification` enum is used to specify whether to force a notification or
/// automatically notify when the progress meter changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressMeterNotification {
    Force,
    Auto,
}

/// The `ProgressMeter` struct is used to track the progress of a task.
pub struct ProgressMeter<T: FromPrimitive + Integer + Copy> {
    /// The total number of steps in the task.
    total: T,

    /// The current step in the task.
    current: T,

    /// The last percentage notified.
    past_percent: T,

    /// The callback function to call when the progress changes.
    callback: Callback<T>,
}

impl<T: FromPrimitive + Integer + Copy> ProgressMeter<T> {
    /// Create a new `ProgressMeter` with the specified total and callback function.
    ///
    /// # Arguments
    ///
    /// * `total` - The total number of steps in the task.
    /// * `callback` - The callback function to call when the progress changes.
    pub fn new(total: T, callback: Callback<T>) -> Self {
        Self {
            total,
            current: T::zero(),
            past_percent: T::zero(),
            callback,
        }
    }

    /// Set the total number of steps in the task.
    ///
    /// # Arguments
    ///
    /// * `total` - The total number of steps in the task.
    pub fn set_total(&mut self, total: T) {
        self.total = total;
    }

    /// Set the callback function to call when the progress changes.
    ///
    /// # Arguments
    ///
    /// * `callback` - The callback function to call when the progress changes.
    pub fn set_callback(&mut self, callback: Callback<T>) {
        self.callback = callback;
    }

    /// Increment the current step in the task by one.
    pub fn increment(&mut self) {
        self.current = self.current + T::one();
    }

    /// Increment the current step in the task by the specified amount.
    ///
    /// # Arguments
    ///
    /// * `amount` - The amount by which to increment the steps.
    pub fn increment_by(&mut self, amount: T) {
        self.current = self.current + amount;
    }

    /// Reset the current step in the task to zero.
    pub fn reset(&mut self) {
        self.current = T::zero();
    }

    /// Notify the callback function of the progress.
    ///
    /// # Arguments
    ///
    /// * `notify` - The notification type to use.
    pub fn notify(&mut self, notify: ProgressMeterNotification) {
        if self.current > self.total {
            self.current = self.total;
        }

        let percent = self.current * T::from_i32(100).unwrap() / self.total;
        if percent != self.past_percent || notify == ProgressMeterNotification::Force {
            self.past_percent = percent;
            (*self.callback)(percent);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::result::DynResult;
    use crate::threadpool::{ThreadJob, ThreadPool};
    use std::sync::{Arc, Mutex};
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_progress_meter() -> DynResult<()> {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let callback: Callback<i32> = Box::pin(move |percent| {
            if let Ok(mut counter) = counter_clone.lock() {
                *counter = percent;
            }
        });
        let progress_meter = Arc::new(Mutex::new(ProgressMeter::new(100, callback)));

        let progress_meter_clone = progress_meter.clone();
        let res = tokio::spawn(async move {
            if let Ok(mut progress_meter) = progress_meter_clone.lock() {
                for _ in 0..100 {
                    progress_meter.increment();
                    progress_meter.notify(ProgressMeterNotification::Auto);
                }
            }
        });

        res.await?;

        if let Ok(counter) = counter.lock() {
            assert_eq!(*counter, 100);
        } else {
            assert!(false);
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_progress_meter_in_threadpool() {
        let counter = Arc::new(Mutex::new(0));
        let counter_clone = counter.clone();
        let callback: Callback<i32> = Box::pin(move |percent| {
            if let Ok(mut counter) = counter_clone.lock() {
                *counter = percent;
            }
        });

        let mut thread_pool = ThreadPool::new(2);
        let mut thread_job1 = ThreadJob::new();

        let meter = Arc::new(Mutex::new(ProgressMeter::new(100, callback)));
        let meter_clone = meter.clone();

        thread_job1.add_task(Box::pin(async move {
            if let Ok(mut meter) = meter_clone.lock() {
                for _ in 0..100 {
                    meter.increment();
                    meter.notify(ProgressMeterNotification::Auto);
                }
            }
            Ok(())
        }));

        if let Err(e) = thread_pool.add_job(thread_job1) {
            panic!("Error adding job: {:?}", e);
        }

        sleep(Duration::from_millis(200)).await;

        thread_pool.stop();

        if let Ok(counter) = counter.lock() {
            assert_eq!(*counter, 100);
        } else {
            assert!(false);
        };
    }

    #[tokio::test]
    async fn test_progress_meter_with_channels() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<i32>();
        let mut progress_meter = ProgressMeter::new(100,
            Box::pin(move |percent| {
                tx.send(percent).unwrap();
            })
        );
        progress_meter.increment();
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 1);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 11);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 21);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 31);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 41);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 51);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 61);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 71);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 81);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 91);
        progress_meter.increment_by(10);
        progress_meter.notify(ProgressMeterNotification::Auto);
        assert_eq!(rx.recv().await.unwrap(), 100);
    }
}
