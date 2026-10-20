//! The `receiver` module provides the [`Receiver`] object used to receive messages from the message
//! channel. The [`Receiver`] object can function as a receiver for either a bounded or unbounded
//! channel.

use crate::multiqueue::MultiQueue;
use crate::sync::mpmc::channel::{Channel, WhichWaker};
use log::error;
use std::future::poll_fn;
use std::mem::ManuallyDrop;
use std::sync::{Arc, Mutex};
use std::task::Poll;
use uuid::Uuid;

/// The receiver object ot use for receiving messages from the channel.
pub struct Receiver<T: Clone> {
    // The actual shared channel.
    channel: Arc<Mutex<Channel<T>>>,

    // A unique id used to associate waker objects with the receiver.
    id: Uuid,

    // A fork of the queue in the channel. This fork allows the receiver to read
    // messages independently of any other fork of the queue. The receiver drops it
    // itself, before it wakes the senders, so that the space it held is free when
    // they look.
    queue: ManuallyDrop<MultiQueue<T>>,
}

impl<T: Clone> Receiver<T> {
    /// Create a new [`Receiver`] object.
    ///
    /// # Arguments
    ///
    /// * `channel` - The shared [`Channel`] object.
    ///
    /// # Returns
    ///
    /// A new [`Receiver`] that will read messages from the shared channel.
    pub(crate) fn new(channel: Arc<Mutex<Channel<T>>>) -> Receiver<T> {
        let queue = match channel.clone().lock() {
            Ok(mut channel) => match channel.queue.fork() {
                Ok(queue) => queue,
                Err(_e) => {
                    panic!("Failed to fork queue");
                }
            },
            Err(_) => {
                panic!("Failed to lock channel");
            }
        };

        Receiver {
            channel,
            id: Uuid::new_v4(),
            queue: ManuallyDrop::new(queue),
        }
    }

    /// Helper function to check when the channel has messages to read.
    ///
    /// # Returns
    ///
    /// Returns Ok(()) when the channel has messages to read or Err(()) when the
    /// channel has no more messages or cannot access shared resources.
    async fn get_something_to_receive(&mut self) -> Result<(), ()> {
        poll_fn(|cx| {
            return if self.queue.size() > 0 {
                // The channel has messages to read.  Since we have messages, remove any
                // previous waker.
                match self.channel.lock() {
                    Ok(mut channel) => {
                        channel.remove_waker(&self.id.to_string(), WhichWaker::Receiver)
                    }
                    Err(_) => {
                        return Poll::Ready(Err(()));
                    }
                }
                Poll::Ready(Ok(()))
            } else {
                // The channel has no messages.
                match self.channel.lock() {
                    Ok(mut channel) => {
                        if channel.live_senders() == 0 {
                            // If we have no more senders, then stop.
                            return Poll::Ready(Err(()));
                        }

                        // We have senders, insert the latest waker till we have
                        // messages to read.
                        channel.set_waker(
                            self.id.to_string(),
                            cx.waker().clone(),
                            WhichWaker::Receiver,
                        );

                        // Just in case, notify the senders that we could read.
                        channel.wake(WhichWaker::Sender);
                        Poll::Pending
                    }
                    Err(_) => Poll::Ready(Err(())),
                }
            };
        })
        .await
    }

    /// Receive a message from the channel.
    ///
    /// # Return
    ///
    /// Some(msg) when the channel is open and has a message, otherwise None
    /// when the channel has closed.
    pub async fn recv(&mut self) -> Option<T> {
        if self.get_something_to_receive().await.is_err() {
            return None;
        }

        match self.channel.lock() {
            Ok(mut channel) => channel.remove_waker(&self.id.to_string(), WhichWaker::Receiver),
            Err(_) => {
                error!("Unable to lock channel to remove waker");
            }
        }
        match self.queue.front() {
            Some(thing) => {
                let tc = thing.clone();
                self.queue.pop_front();

                // Try to wake a sender
                match self.channel.lock() {
                    Ok(mut channel) => channel.wake(WhichWaker::Sender),
                    Err(_) => {
                        error!("Unable to lock channel to wake sender");
                    }
                }

                Some(tc)
            }
            None => None,
        }
    }
}

impl<T: Clone> Drop for Receiver<T> {
    fn drop(&mut self) {
        // Dropping the fork releases the messages it had not read. In a bounded channel a sender
        // waits while the slowest receiver holds the channel full, and only a wake tells it that
        // the space is free, so wake the senders once the fork is gone.
        unsafe {
            ManuallyDrop::drop(&mut self.queue);
        }
        match self.channel.lock() {
            Ok(mut channel) => {
                channel.remove_waker(&self.id.to_string(), WhichWaker::Receiver);
                channel.wake(WhichWaker::Sender);
            }
            Err(_) => {
                error!("Unable to lock channel to wake senders");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::mpmc::bounded::channel;
    use crate::sync::mpmc::unbounded::unbounded_channel;
    use std::time::Duration;

    /// Poison the mutex guarding a channel so that subsequent lock attempts fail. This drives the
    /// lock-error branches in the receiver. Poisoning happens on the current thread by panicking
    /// while holding the guard.
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
    async fn test_recv_on_poisoned_empty_channel() {
        // With no messages queued and a poisoned channel, recv cannot lock the channel and returns
        // None instead of blocking.
        let (sender, mut receiver) = unbounded_channel::<i32>();
        poison_channel(&receiver.channel);

        assert_eq!(receiver.recv().await, None);
        drop(sender);
    }

    #[tokio::test]
    async fn test_recv_on_poisoned_channel_with_message() {
        // A message is available in the receiver's fork, but the poisoned channel lock prevents
        // clearing the waker, so recv returns None.
        let (sender, mut receiver) = unbounded_channel::<i32>();
        sender.send(5).await.unwrap();
        poison_channel(&receiver.channel);

        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    async fn test_buffered_messages_survive_sender_drop() {
        // Messages already in the channel remain readable after every sender drops; the receiver
        // drains them in FIFO order and only then observes the closed channel as None.
        let (sender, mut receiver) = unbounded_channel::<i32>();

        sender.send(1).await.unwrap();
        sender.send(2).await.unwrap();
        sender.send(3).await.unwrap();
        drop(sender);

        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));
        assert_eq!(receiver.recv().await, Some(3));
        assert_eq!(receiver.recv().await, None);
        // Once closed, the channel stays closed on subsequent reads.
        assert_eq!(receiver.recv().await, None);
    }

    #[tokio::test]
    #[should_panic(expected = "Failed to lock channel")]
    async fn test_subscribe_on_poisoned_channel_panics() {
        // Creating a new receiver against a poisoned channel panics because the queue fork requires
        // locking the channel.
        let (sender, receiver) = unbounded_channel::<i32>();
        poison_channel(&receiver.channel);
        let _receiver2 = sender.subscribe();
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_last_sender_drop_wakes_a_waiting_receiver() {
        // A receiver already waiting for a message returns None once the last sender leaves.
        let (sender, mut receiver) = channel::<i32>(4);
        let waiter = tokio::spawn(async move { receiver.recv().await });
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(sender);
        let received = tokio::time::timeout(Duration::from_secs(2), waiter)
            .await
            .expect("the receiver still waits after the last sender left");
        assert_eq!(received.unwrap(), None);
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_other_senders_keep_a_waiting_receiver_open() {
        // While one sender remains, a waiting receiver keeps waiting, and gets that sender's message.
        let (sender, mut receiver) = channel::<i32>(4);
        let other = sender.clone();
        let waiter = tokio::spawn(async move { receiver.recv().await });
        tokio::time::sleep(Duration::from_millis(200)).await;
        drop(sender);
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!waiter.is_finished());
        other.send(7).await.unwrap();
        let received = tokio::time::timeout(Duration::from_secs(2), waiter).await.unwrap();
        assert_eq!(received.unwrap(), Some(7));
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_receiver_drop_wakes_a_waiting_sender() {
        // A bounded channel stays full while one receiver has not read, so a sender waits. Dropping
        // that receiver frees the space and lets the send finish.
        let (sender, mut reader) = channel::<i32>(1);
        let lagging = sender.subscribe();
        sender.send(1).await.unwrap();
        assert_eq!(reader.recv().await, Some(1));

        let blocked = tokio::spawn(async move {
            sender.send(2).await.unwrap();
            sender
        });
        tokio::time::sleep(Duration::from_millis(200)).await;
        assert!(!blocked.is_finished());

        drop(lagging);
        let sender = tokio::time::timeout(Duration::from_secs(2), blocked)
            .await
            .expect("the sender still waits after the full receiver left")
            .unwrap();
        assert_eq!(reader.recv().await, Some(2));
        drop(sender);
        assert_eq!(reader.recv().await, None);
    }
}
