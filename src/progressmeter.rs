//! The `progressmeter` module provides a simple progress meter for tracking the progress of a
//! long-running task.

/// The `Notifier` type is a type alias for a boxed closure that receives notifications when the
/// progress meter makes progress towards the total goal. The value passed to the function represents
/// the current percent completed out of 100.
pub type Notifier = Box<dyn FnMut(u8) + Send + Sync + 'static>;

/// The `ProgressMeter` struct provides a simple progress meter for tracking the progress of a
/// long-running task. The user provides a notification closure or function that receives notifications
/// when the progress meter makes progress towards the total goal. The progress meter can be
/// incremented by one or by a specified amount, and the user can also reset the progress meter to
/// zero.
pub struct ProgressMeter {
    /// The notification function that receives calls when the progress meter makes progress towards
    /// the total goal (in percentage terms). The value passed to the function represents the current
    /// percent completed out of 100.
    notifier: Notifier,

    /// The total number of units that the progress meter is tracking.
    meter_total: u64,

    /// The current number of units that the progress meter has tracked.
    meter_current: u64,

    /// The last percentage that was notified to the user.
    last_percent: u8,
}

impl Default for ProgressMeter {
    fn default() -> Self {
        Self::new()
    }
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
    ///   progress towards the total goal (in percentage terms). The value passed to the function
    ///   represents the current percent completed out of 100.
    /// * `meter_total` - The total number of units that the progress meter is tracking.
    ///
    /// # Returns
    ///
    /// A new `ProgressMeter` with the given notifier function and total number of units to track.
    pub fn new_with_notifier_and_size(notifier: Notifier, meter_total: u64) -> ProgressMeter {
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
    ///   changed since the last notification.
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

    /// Set the current number of units that the progress meter has tracked.
    ///
    /// If the current unit is larger than the amount the tracker is tracking, then
    /// the method will set the current units to the total units being tracked.
    ///
    /// # Arguments
    ///
    /// * `current` - The current number of units that the progress meter has tracked.
    pub fn set_current(&mut self, current: u64) {
        if current > self.meter_total {
            self.meter_current = self.meter_total;
        } else {
            self.meter_current = current;
        }
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
    ///   progress towards the total goal (in percentage terms). The value passed to the function
    ///   represents the current percent completed out of 100.
    pub fn set_notifier(&mut self, notifier: Notifier) {
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

    #[tokio::test]
    async fn test_progress_meter_set_current() {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<u8>();
        let mut progress_meter = ProgressMeter::new_with_notifier_and_size(
            Box::new(move |percent| {
                tx.send(percent).unwrap();
            }),
            100,
        );
        progress_meter.set_current(10);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 10);

        progress_meter.set_current(50);
        progress_meter.notify(false);
        assert_eq!(rx.recv().await.unwrap(), 50);
    }

    /// Records every percentage value passed to the notifier, allowing tests to
    /// assert both the values reported and how many times the notifier fired.
    fn recording_meter(total: u64) -> (ProgressMeter, std::sync::Arc<std::sync::Mutex<Vec<u8>>>) {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let log_clone = log.clone();
        let meter = ProgressMeter::new_with_notifier_and_size(
            Box::new(move |percent| {
                log_clone.lock().unwrap().push(percent);
            }),
            total,
        );
        (meter, log)
    }

    #[test]
    fn test_new_defaults() {
        // The default meter tracks a single unit, so one increment is 100%.
        let mut meter = ProgressMeter::new();
        meter.increment();
        // The default notifier is a no-op; this exercises it without panicking.
        meter.notify(true);
    }

    #[test]
    fn test_new_default_notifier_is_noop() {
        // Drive the default no-op notifier across several updates to ensure it
        // never panics regardless of the force flag.
        let mut meter = ProgressMeter::new();
        meter.set_total(10);
        meter.increment_by(5);
        meter.notify(false);
        meter.notify(true);
        meter.reset();
        meter.notify(true);
    }

    #[test]
    fn test_reset_returns_to_zero() {
        let (mut meter, log) = recording_meter(100);
        meter.increment_by(40);
        meter.notify(false);
        meter.reset();
        // After a reset the percentage drops back to 0. Because 0 is not greater
        // than the last notified percentage (40), a non-forced notify is silent.
        meter.notify(false);
        meter.notify(true);
        assert_eq!(*log.lock().unwrap(), vec![40, 0]);
    }

    #[test]
    fn test_notify_skips_when_percent_not_increased() {
        let (mut meter, log) = recording_meter(100);
        meter.set_current(30);
        meter.notify(false);
        // No progress made: percent is unchanged, so a non-forced notify is a no-op.
        meter.notify(false);
        // Progress that does not cross a whole-percent boundary is also silent.
        meter.set_current(30);
        meter.notify(false);
        assert_eq!(*log.lock().unwrap(), vec![30]);
    }

    #[test]
    fn test_notify_force_repeats_same_percent() {
        let (mut meter, log) = recording_meter(100);
        meter.set_current(25);
        meter.notify(false);
        // Forcing re-notifies the same percentage even though it has not changed.
        meter.notify(true);
        meter.notify(true);
        assert_eq!(*log.lock().unwrap(), vec![25, 25, 25]);
    }

    #[test]
    fn test_notify_clamps_current_above_total() {
        let (mut meter, log) = recording_meter(100);
        // Incrementing beyond the total clamps to 100% rather than overflowing.
        meter.increment_by(250);
        meter.notify(false);
        assert_eq!(*log.lock().unwrap(), vec![100]);
    }

    #[test]
    fn test_set_current_clamps_above_total() {
        let (mut meter, log) = recording_meter(50);
        // set_current saturates at the total, so this reports 100%, not 200%.
        meter.set_current(100);
        meter.notify(false);
        assert_eq!(*log.lock().unwrap(), vec![100]);
    }

    #[test]
    fn test_set_total_rescales_percentage() {
        let (mut meter, log) = recording_meter(100);
        meter.set_current(10);
        meter.notify(false);
        // Shrinking the total makes the same current count a larger fraction.
        meter.set_total(20);
        meter.notify(false);
        assert_eq!(*log.lock().unwrap(), vec![10, 50]);
    }

    #[test]
    fn test_set_notifier_replaces_callback() {
        // Start with the no-op default, then install a recording notifier.
        let mut meter = ProgressMeter::new();
        meter.set_total(100);
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::<u8>::new()));
        let log_clone = log.clone();
        meter.set_notifier(Box::new(move |percent| {
            log_clone.lock().unwrap().push(percent);
        }));
        meter.set_current(75);
        meter.notify(false);
        assert_eq!(*log.lock().unwrap(), vec![75]);
    }
}
