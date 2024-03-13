//! The `receiver` module provides the [`Receiver`] object used to receive messages from the message
//! channel. The [`Receiver`] object can function as a receiver for either a bounded or unbounded
//! channel.

use crate::multiqueue::MultiQueue;
use crate::sync::mpmc::channel::{Channel, WhichWaker};
use log::error;
use std::future::poll_fn;
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
    // messages independently of any other fork of the queue.
    queue: MultiQueue<T>,
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
            queue,
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
        if let Err(_) = self.get_something_to_receive().await {
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
