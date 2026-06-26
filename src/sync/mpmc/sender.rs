//! The `sender` module provides the [`Sender`] object used to send messages to the message channel.
//! The [`Sender`] object can function as a sender for either a bounded channel or an unbounded
//! channel.

use crate::sync::error::SendError;
use crate::sync::mpmc::channel::{Channel, WhichWaker};
use crate::sync::mpmc::receiver::Receiver;
use log::error;
use std::future::poll_fn;
use std::sync::{Arc, Mutex};
use std::task::Poll;
use uuid::Uuid;

/// The sender object to use for sending messages to the channel.
pub struct Sender<T: Clone> {
    /// The actual shared channel object.
    channel: Arc<Mutex<Channel<T>>>,

    /// Some(n) indicates that the channel allows n messages in
    /// the channel, None indicates the channel is unbounded.
    bound: Option<usize>,

    /// A unique identifier used to associate tokio [`Waker`] objects
    /// with this sender.
    id: Uuid,
}

/// A helper function to increment the number of senders used by the channel.
///
/// # Arguments
///
/// * `channel` - A shared channel object.
pub(crate) fn increment_senders<T: Clone>(channel: &Arc<Mutex<Channel<T>>>) {
    match channel.lock() {
        Ok(mut channel) => {
            channel.increment_senders();
        }
        Err(_) => {
            error!("Unable to increment channel sender count");
        }
    }
}

impl<T: Clone> Sender<T> {
    /// Create a new bounded or unbounded sender.
    ///
    /// # Arguments
    ///
    /// * `channel` - The shared [`Channel`] object.
    /// * `bound` - Some(n) indicates that the channel should have no more than n messages in the
    ///   channel and None indicates and unbounded channel.
    ///
    /// # Returns
    ///
    /// A new [`Sender`].
    pub(crate) fn new(channel: Arc<Mutex<Channel<T>>>, bound: Option<usize>) -> Sender<T> {
        increment_senders(&channel);

        Sender {
            channel,
            bound,
            id: Uuid::new_v4(),
        }
    }

    /// A helper function that performs the work of a Future on behalf of the [`Sender::send`]
    /// function.
    ///
    /// # Returns
    ///
    /// A result of Ok(()) when the channel has space to send a message and an Err(SendError(()))
    /// when an error occurs.
    async fn get_send_space(&self) -> Result<(), SendError<()>> {
        poll_fn(|cx| match self.channel.lock() {
            Ok(mut channel) => {
                // Check for the bounded case.
                if let Some(bound) = self.bound {
                    if channel.queue.shared_size() < bound {
                        return Poll::Ready(Ok(()));
                    }

                    // Here we do not have space.  Send a wake notice to any receivers before we return
                    // pending.
                    channel.wake(WhichWaker::Receiver);

                    channel.set_waker(self.id.to_string(), cx.waker().clone(), WhichWaker::Sender);
                    Poll::Pending
                } else {
                    // In the unbounded case, we are always ready.
                    Poll::Ready(Ok(()))
                }
            }
            Err(_e) => Poll::Ready(Err(SendError(()))),
        })
        .await
    }

    /// Send a message to the channel.
    ///
    /// Unlike other senders in other kinds of message channels, this channel uses a Future
    /// to know when the channel has room for more messages. The user should `await` the
    /// send call in order to allow receivers the opportunity to remove messages prior to
    /// sending the next message.
    ///
    /// # Arguments
    ///
    /// * `thing` - the message
    ///
    /// # Returns
    ///
    /// A result of Ok(()) if the send operation succeeds and Err(SendError(thing)) if the send fails.
    pub async fn send(&self, thing: T) -> Result<(), SendError<T>> {
        // The 'await' happens here, so we do not pass thing into a closure that then needs
        // to go into the channel.send() function.
        if self.get_send_space().await.is_err() {
            return Err(SendError(thing));
        }
        match self.channel.lock() {
            Ok(mut channel) => {
                channel.send(thing)?;
                channel.remove_waker(&self.id.to_string(), WhichWaker::Sender);
                channel.wake(WhichWaker::Receiver);
            }
            Err(_e) => {
                return Err(SendError(thing));
            }
        }
        Ok(())
    }

    /// Create a new [`Receiver`] that will receive all the messages in the channel after
    /// the function returns.
    ///
    /// # Returns
    ///
    /// A new [`Receiver`] object that will receive messages from the channel.
    pub fn subscribe(&self) -> Receiver<T> {
        Receiver::new(self.channel.clone())
    }
}

impl<T: Clone> Clone for Sender<T> {
    /// Clone the sending channel to create multiple senders.
    ///
    /// The returned sender shares the channel with the requester.
    fn clone(&self) -> Sender<T> {
        increment_senders(&self.channel);
        Sender {
            channel: self.channel.clone(),
            bound: self.bound,
            id: Uuid::new_v4(),
        }
    }
}

impl<T: Clone> Drop for Sender<T> {
    fn drop(&mut self) {
        match self.channel.lock() {
            Ok(mut channel) => {
                channel.decrement_senders();
            }
            Err(_) => {
                error!("Unable to decrement channel senders");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::mpmc::bounded::channel;
    use crate::sync::mpmc::unbounded::unbounded_channel;

    /// Poison the mutex guarding a channel so that subsequent lock attempts fail. This lets us
    /// exercise the lock-error branches in the sender. Poisoning happens on the current thread by
    /// panicking while holding the guard.
    fn poison_channel<T>(channel: &Arc<Mutex<Channel<T>>>) {
        let handle = channel.clone();
        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
            let _guard = handle.lock().unwrap();
            panic!("intentionally poison the mutex for testing");
        }));
        std::panic::set_hook(previous_hook);
        assert!(channel.is_poisoned());
    }

    #[tokio::test]
    async fn test_clone_shares_channel() {
        // A cloned sender shares the same channel; messages sent through it reach the receiver and
        // both senders keep the channel open.
        let (sender, mut receiver) = unbounded_channel::<i32>();
        let sender2 = sender.clone();

        {
            let channel = sender.channel.lock().unwrap();
            assert_eq!(channel.live_senders(), 2);
        }

        sender2.send(10).await.unwrap();
        sender.send(11).await.unwrap();

        assert_eq!(receiver.recv().await, Some(10));
        assert_eq!(receiver.recv().await, Some(11));
    }

    #[tokio::test]
    async fn test_clone_with_poisoned_channel() {
        // Cloning still produces a sender even when the channel lock is poisoned (the sender count
        // simply cannot be incremented).
        let (sender, _receiver) = unbounded_channel::<i32>();
        poison_channel(&sender.channel);

        let cloned = sender.clone();
        assert!(cloned.channel.is_poisoned());
        // Dropping the senders here exercises the poisoned-lock branch in Drop.
    }

    #[tokio::test]
    async fn test_send_on_poisoned_channel_errors() {
        // Sending on a poisoned channel returns the original message back in the SendError.
        let (sender, _receiver) = unbounded_channel::<i32>();
        poison_channel(&sender.channel);

        let result = sender.send(99).await;
        assert!(matches!(result, Err(SendError(99))));
    }

    #[tokio::test]
    async fn test_drop_on_poisoned_channel() {
        // Dropping a sender whose channel lock is poisoned must not panic.
        let (sender, _receiver) = unbounded_channel::<i32>();
        poison_channel(&sender.channel);
        drop(sender);
    }

    #[tokio::test]
    async fn test_subscribe_receives_only_future_messages() {
        // A receiver created via subscribe forks the queue at the moment of subscription, so it
        // only sees messages sent afterward, while the original receiver sees everything.
        let (sender, mut receiver) = unbounded_channel::<i32>();

        sender.send(1).await.unwrap();
        let mut late = sender.subscribe();
        sender.send(2).await.unwrap();

        // The original receiver observes both messages.
        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));

        // The late subscriber only observes the message sent after it subscribed.
        assert_eq!(late.recv().await, Some(2));

        // No more senders once dropped: both receivers should now drain to None.
        drop(sender);
        assert_eq!(receiver.recv().await, None);
        assert_eq!(late.recv().await, None);
    }

    #[tokio::test]
    async fn test_dropping_all_senders_closes_channel() {
        // Once every sender drops, a receiver that has drained the channel receives None.
        let (sender, mut receiver) = unbounded_channel::<i32>();
        let sender2 = sender.clone();

        sender.send(1).await.unwrap();
        sender2.send(2).await.unwrap();

        // Dropping only one sender leaves the channel open.
        drop(sender);
        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));

        // Dropping the last sender closes the channel.
        drop(sender2);
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn test_bounded_channel_applies_backpressure() {
        // A bounded channel only permits `bound` unread messages; the third send cannot complete
        // until a receiver reads one out, so it stays pending until then.
        let (sender, mut receiver) = channel::<i32>(2);

        sender.send(1).await.unwrap();
        sender.send(2).await.unwrap();

        // The queue is full; spawn the third send and confirm it does not complete on its own.
        let send_handle = tokio::spawn(async move {
            sender.send(3).await.unwrap();
            sender
        });

        // Give the spawned send a chance to run; it must remain pending while the channel is full.
        tokio::task::yield_now().await;
        assert!(!send_handle.is_finished());

        // Reading a message frees a slot and lets the pending send complete.
        assert_eq!(receiver.recv().await, Some(1));
        let sender = send_handle.await.unwrap();

        assert_eq!(receiver.recv().await, Some(2));
        assert_eq!(receiver.recv().await, Some(3));

        drop(sender);
        assert_eq!(receiver.recv().await, None);
    }
}
