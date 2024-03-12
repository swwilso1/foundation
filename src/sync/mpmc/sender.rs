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
    /// channel and None indicates and unbounded channel.
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

    /// Clone the sending channel to create multiple senders.
    ///
    /// # Returns
    ///
    /// A sender that shares the channel with the requester.
    pub fn clone(&self) -> Sender<T> {
        increment_senders(&self.channel);
        Sender {
            channel: self.channel.clone(),
            bound: self.bound,
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
        if let Err(_) = self.get_send_space().await {
            return Err(SendError(thing));
        }
        match self.channel.lock() {
            Ok(mut channel) => {
                channel.send(thing)?;
                channel.senders.remove(&self.id.to_string());
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
