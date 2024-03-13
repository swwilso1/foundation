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

    #[test]
    fn test_channel() {
        let mut channel: Channel<i32> = Channel::new();
        channel.send(1).unwrap();
        channel.send(2).unwrap();
        channel.send(3).unwrap();
    }
}
