//! The `channel` module provides the lower level shared object for a mpmc message channel.

use crate::multiqueue::{MultiQueue, MultiQueueError};
use crate::sync::error::SendError;
use std::collections::HashMap;
use std::task::Waker;

#[derive(Debug, Clone)]
/// A simple enum that indicates whether to operate on sender wakers or receiver wakers.
pub(crate) enum WhichWaker {
    Sender,
    Receiver,
}

/// The shared channel object.
pub(crate) struct Channel<T> {
    /// A map of sender ids to the wakers used to wake the senders.
    senders: HashMap<String, Waker>,

    /// A map of receiver ids to the wakers used to wake the receivers.
    receivers: HashMap<String, Waker>,

    /// The original shared message queue.  Receivers have a fork of this
    /// queue.
    pub queue: MultiQueue<T>,

    /// The number of remaining non-dropped senders.  When this value goes to
    /// zero, the channel is effectively 'closed'.
    live_senders: usize,
}

impl<T> Channel<T> {
    /// Create a new [`Channel`]
    pub fn new() -> Channel<T> {
        Channel {
            senders: HashMap::new(),
            receivers: HashMap::new(),
            queue: MultiQueue::new(),
            live_senders: 0,
        }
    }

    /// Send a message by appending the message to the queue.
    /// This function manages the details of handling the queue and
    /// signalling any receiver wakers waiting for a notification to
    /// read the queue again.
    pub fn send(&mut self, thing: T) -> Result<(), SendError<T>> {
        // Note we do not check bounds here because that should be handled in the bounded
        // or unbounded code.
        match self.queue.push_back(thing) {
            Ok(()) => {
                // Make sure we drain our read side of the queue.  Remember, the receiver contains
                // a fork of the queue. In order to bound memory usage, we need the receivers to
                // be the only objects holding references to the queue data.
                self.queue.pop_all();

                Ok(())
            }
            Err(e) => {
                match e {
                    MultiQueueError::Push(thing) => Err(SendError(thing)),
                    _ => {
                        // TODO: Revisit this error condition.
                        panic!("push_back should not return anything other than MultiQueueError::Push(T).");
                    }
                }
            }
        }
    }

    /// Decrement the count of the number of senders.
    pub fn decrement_senders(&mut self) {
        self.live_senders -= 1;
    }

    /// Increment the count of the number of senders.
    pub fn increment_senders(&mut self) {
        self.live_senders += 1;
    }

    /// Return the current number of active senders.
    pub fn live_senders(&self) -> usize {
        self.live_senders
    }

    /// A helper function to return the map for either the senders or receivers.
    ///
    /// # Arguments
    ///
    /// * `which` - the enum indicating either sender or receiver.
    ///
    /// # Returns
    ///
    /// A reference to either the sender map or the receiver map.
    fn which_table(&mut self, which: WhichWaker) -> &mut HashMap<String, Waker> {
        match which {
            WhichWaker::Sender => &mut self.senders,
            WhichWaker::Receiver => &mut self.receivers,
        }
    }

    /// Set an id, waker pair in either the sender or receiver map.
    ///
    /// # Arguments
    ///
    /// * `id` - the id of the object setting the waker
    /// * `waker` - the [`Waker`]
    /// * `which` - enum indicating whether to use the sender or receiver map.
    pub fn set_waker(&mut self, id: String, waker: Waker, which: WhichWaker) {
        let table = self.which_table(which);
        table.insert(id, waker);
    }

    /// Notify all the wakers in either the sender or receiver maps.
    ///
    /// # Arguments
    ///
    /// * `which` - An enum indicating whether to use the sender or receiver maps.
    pub fn wake(&mut self, which: WhichWaker) {
        let table = self.which_table(which);
        for (_id, waker) in table {
            waker.wake_by_ref();
        }
    }

    /// Remove an id/waker pair from either the sender or receiver maps.
    ///
    /// # Arguments
    ///
    /// * `id` - the identification of the waker to remove.
    /// * `which` - An enum indicating whether to use the sender or receiver maps.
    pub fn remove_waker(&mut self, id: &str, which: WhichWaker) {
        let table = self.which_table(which);
        table.remove(id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::task::Wake;

    /// A waker that counts how many times it gets woken. This lets the waker-management tests
    /// observe exactly which wakers `Channel::wake` notifies.
    struct CountingWaker {
        count: Arc<AtomicUsize>,
    }

    impl Wake for CountingWaker {
        fn wake(self: Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }

        fn wake_by_ref(self: &Arc<Self>) {
            self.count.fetch_add(1, Ordering::SeqCst);
        }
    }

    /// Build a [`Waker`] together with the counter it increments when woken.
    fn counting_waker() -> (Waker, Arc<AtomicUsize>) {
        let count = Arc::new(AtomicUsize::new(0));
        let waker = Waker::from(Arc::new(CountingWaker {
            count: count.clone(),
        }));
        (waker, count)
    }

    #[test]
    fn test_channel() {
        let mut channel: Channel<i32> = Channel::new();
        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.send(3).unwrap();
    }

    #[test]
    fn test_sender_count_tracking() {
        // A fresh channel starts with no senders, and increment/decrement track the live count.
        let mut channel: Channel<i32> = Channel::new();
        assert_eq!(channel.live_senders(), 0);

        channel.increment_senders();
        channel.increment_senders();
        assert_eq!(channel.live_senders(), 2);

        channel.decrement_senders();
        assert_eq!(channel.live_senders(), 1);

        channel.decrement_senders();
        assert_eq!(channel.live_senders(), 0);
    }

    #[test]
    fn test_wake_only_notifies_requested_table() {
        // Wakers live in separate sender and receiver tables; waking one table must not touch the
        // other.
        let mut channel: Channel<i32> = Channel::new();
        let (sender_waker, sender_count) = counting_waker();
        let (receiver_waker, receiver_count) = counting_waker();

        channel.set_waker("s1".to_string(), sender_waker, WhichWaker::Sender);
        channel.set_waker("r1".to_string(), receiver_waker, WhichWaker::Receiver);

        channel.wake(WhichWaker::Receiver);
        assert_eq!(receiver_count.load(Ordering::SeqCst), 1);
        assert_eq!(sender_count.load(Ordering::SeqCst), 0);

        channel.wake(WhichWaker::Sender);
        assert_eq!(receiver_count.load(Ordering::SeqCst), 1);
        assert_eq!(sender_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_remove_waker_stops_notifications() {
        // After removing a waker it no longer gets notified, and re-inserting under the same id
        // replaces the previous waker rather than keeping both.
        let mut channel: Channel<i32> = Channel::new();
        let (waker, count) = counting_waker();

        channel.set_waker("r1".to_string(), waker, WhichWaker::Receiver);
        channel.remove_waker("r1", WhichWaker::Receiver);
        channel.wake(WhichWaker::Receiver);
        assert_eq!(count.load(Ordering::SeqCst), 0);

        // Inserting two wakers under the same id keeps only the last one.
        let (first, first_count) = counting_waker();
        let (second, second_count) = counting_waker();
        channel.set_waker("dup".to_string(), first, WhichWaker::Receiver);
        channel.set_waker("dup".to_string(), second, WhichWaker::Receiver);
        channel.wake(WhichWaker::Receiver);
        assert_eq!(first_count.load(Ordering::SeqCst), 0);
        assert_eq!(second_count.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn test_remove_missing_waker_is_noop() {
        // Removing an id that was never registered does nothing and does not panic.
        let mut channel: Channel<i32> = Channel::new();
        channel.remove_waker("does-not-exist", WhichWaker::Sender);
        channel.remove_waker("does-not-exist", WhichWaker::Receiver);
    }

    #[test]
    fn test_which_waker_derives() {
        // WhichWaker derives Clone and Debug; exercise both variants.
        let sender = WhichWaker::Sender;
        let receiver = WhichWaker::Receiver;
        assert_eq!(format!("{:?}", sender.clone()), "Sender");
        assert_eq!(format!("{:?}", receiver.clone()), "Receiver");
    }
}
