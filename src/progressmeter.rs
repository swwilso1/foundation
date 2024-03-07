//! The `progressmeter` module provides a simple progress meter for tracking the progress of a
//! long-running task.

/// The `ProgressMeter` struct provides a simple progress meter for tracking the progress of a
/// long-running task. The user provides a notification closure or function that receives notifications
/// when the progress meter makes progress towards the total goal. The progress meter can be
/// incremented by one or by a specified amount, and the user can also reset the progress meter to
/// zero.
pub struct ProgressMeter {
    /// The notification function that receives calls when the progress meter makes progress towards
    /// the total goal (in percentage terms). The value passed to the function represents the current
    /// percent completed out of 100.
    notifier: Box<dyn FnMut(u8) -> ()>,

    /// The total number of units that the progress meter is tracking.
    meter_total: u64,

    /// The current number of units that the progress meter has tracked.
    meter_current: u64,

    /// The last percentage that was notified to the user.
    last_percent: u8,
}

impl ProgressMeter {
    /// Create a new `ProgressMeter` with the default notifier function and a total number of units
    /// to track of 1.
    pub fn new() -> ProgressMeter {
        ProgressMeter {
            notifier: Box::new(|_| {}),
            meter_total: 1,
            meter_current: 0,
            last_percent: 0,
        }
    }

    /// Create a new `ProgressMeter` with the given notifier function and total number of units to
    /// track.
    ///
    /// # Arguments
    ///
    /// * `notifier` - The notification function that receives calls when the progress meter makes
    /// progress towards the total goal (in percentage terms). The value passed to the function
    /// represents the current percent completed out of 100.
    /// * `meter_total` - The total number of units that the progress meter is tracking.
    ///
    /// # Returns
    ///
    /// A new `ProgressMeter` with the given notifier function and total number of units to track.
    pub fn new_with_notifier_and_size(
        notifier: Box<dyn FnMut(u8) -> ()>,
        meter_total: u64,
    ) -> ProgressMeter {
        ProgressMeter {
            notifier,
            meter_total,
            meter_current: 0,
            last_percent: 0,
        }
    }

    /// Increment the progress meter by one unit.
    pub fn increment(&mut self) {
        self.meter_current += 1;
    }

    /// Increment the progress meter by the given amount.
    ///
    /// # Arguments
    ///
    /// * `increment` - The amount to increment the progress meter by.
    pub fn increment_by(&mut self, increment: u64) {
        self.meter_current += increment;
    }

    /// Reset the progress meter to zero.
    pub fn reset(&mut self) {
        self.meter_current = 0;
    }

    /// Notify the user of the current progress of the progress meter. If the force flag is set to
    /// true, the notification function will be called even if the progress has not changed since the
    /// last notification.
    ///
    /// # Arguments
    ///
    /// * `force` - A flag indicating whether to force a notification even if the progress has not
    /// changed since the last notification.
    pub fn notify(&mut self, force: bool) {
        if self.meter_current > self.meter_total {
            self.meter_current = self.meter_total;
        }

        let percent = ((self.meter_current as f64 / self.meter_total as f64) * 100.0) as u8;
        if percent > self.last_percent || force {
            (self.notifier)(percent);
        }
        self.last_percent = percent;
    }

    /// Set the total number of units that the progress meter is tracking.
    pub fn set_total(&mut self, total: u64) {
        self.meter_total = total;
    }

    /// Set the notifier function that receives calls when the progress meter makes progress towards
    /// the total goal (in percentage terms). The value passed to the function represents the current
    /// percent completed out of 100.
    ///
    /// # Arguments
    ///
    /// * `notifier` - The notification function that receives calls when the progress meter makes
    /// progress towards the total goal (in percentage terms). The value passed to the function
    /// represents the current percent completed out of 100.
    pub fn set_notifier(&mut self, notifier: Box<dyn FnMut(u8) -> ()>) {
        self.notifier = notifier;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_progress_meter() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<u8>();
        let mut progress_meter = ProgressMeter::new_with_notifier_and_size(
            Box::new(move |percent| {
                tx.send(percent).unwrap();
            }),
            100,
        );
        progress_meter.increment();
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 1);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 11);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 21);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 31);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 41);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 51);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 61);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 71);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 81);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 91);
        progress_meter.increment_by(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 100);
    }
}
