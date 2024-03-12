//! The `mpmc` module provides two multiple producer, multiple consumer type channels.
//! Each item sent by a [`Sender`] will arrive in order at the [`Receiver`] end of the channel.
//! These channels guarantee that all messages sent by the producers will arrive at the
//! consumers, nothing will get lost.
//!
//! When a value gets sent, **all** [`Receiver`] handles get notified and will receive the value.
//! The channel stores the value once inside the channel and clones the value for each [`Receiver`].
//! Once all receivers have received a clone of the value, the channel releases the message.
//!
//! The module provides two versions of the channel, a bounded channel and an unbounded channel. The
//! bounded channel places a maximum limit on the number of messages in the channel at any given
//! time. Unlike other asynchronous message channels, when the sender tries to send a message to
//! a full channel, the send operation will not return until the receivers have all read the
//! next message in the queue. The unbounded channel does not have any bounds on the number of
//! messages in the channel (up to the limit of memory in the device).
//!
//! To create a channel, call the [`channel`] function and provide a bound as the function argument
//! or call the [`unbounded_channel`] function to create an unbounded channel. Unlike other async
//! channels, both the [`channel`] function and the [`unbounded_channel`] function return the same
//! pair of types, a tuple of [`Sender`] and [`Receiver`]. Both channels also work well in the
//! single producer, multiple consumer case where a single producer sends messages to many
//! consumers.
//!
//! To create multiple consumers, call the [`Sender::subscribe`] method and each [`Receiver`]
//! created by the call will receive a copy of all the messages in the channel from the time of the
//! call to `subscribe`.
//!
//! ## Closing
//!
//! When *all* [`Sender`] handles have dropped, the channel has "closed". Once a receiver has
//! received all the values in the channel, the next call to [`Receiver::recv`] will return `None`.
//!
//! When the code drops a [`Receiver`] handle, any messages remaining in the channel get marked
//! as read. When the last receiver drops, then the [`Receiver`] code will drop any remaining
//! messages.
//!
//! [`Sender`]: crate::sync::mpmc::sender::Sender
//! [`Sender::subscribe`]: crate::sync::mpmc::sender::Sender::subscribe
//! [`Receiver`]: crate::sync::mpmc::receiver::Receiver
//! [`Receiver::recv`]: crate::sync::mpmc::receiver::Receiver::recv
//! [`channel`]: crate::sync::mpmc::bounded::channel
//! [`unbounded_channel`]: crate::sync::mpmc::unbounded::unbounded_channel
//!
//! # Examples
//!
//! Basic bounded usage
//!
//! ```rust
//! use foundation::sync::mpmc::bounded;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (sender, mut receiver) = bounded::channel(2);
//!     let mut receiver2 = sender.subscribe();
//!
//!     tokio::spawn(async move {
//!        assert_eq!(receiver.recv().await.unwrap(), 10);
//!        assert_eq!(receiver.recv().await.unwrap(), 11);
//!        assert_eq!(receiver.recv().await.unwrap(), 12);
//!     });
//!
//!     tokio::spawn(async move {
//!        assert_eq!(receiver2.recv().await.unwrap(), 10);
//!        assert_eq!(receiver2.recv().await.unwrap(), 11);
//!        assert_eq!(receiver2.recv().await.unwrap(), 12);
//!     });
//!
//!     sender.send(10).await.unwrap();
//!     sender.send(11).await.unwrap();
//!     sender.send(12).await.unwrap();
//! }
//! ```
//!
//! ```rust
//! use foundation::sync::mpmc::unbounded;
//!
//! #[tokio::main]
//! async fn main() {
//!     let (sender, mut receiver) = unbounded::unbounded_channel();
//!     let mut receiver2 = sender.subscribe();
//!
//!     tokio::spawn(async move {
//!        assert_eq!(receiver.recv().await.unwrap(), 10);
//!        assert_eq!(receiver.recv().await.unwrap(), 11);
//!        assert_eq!(receiver.recv().await.unwrap(), 12);
//!     });
//!
//!     tokio::spawn(async move {
//!        assert_eq!(receiver2.recv().await.unwrap(), 10);
//!        assert_eq!(receiver2.recv().await.unwrap(), 11);
//!        assert_eq!(receiver2.recv().await.unwrap(), 12);
//!     });
//!
//!     sender.send(10).await.unwrap();
//!     sender.send(11).await.unwrap();
//!     sender.send(12).await.unwrap();
//! }
//! ```

pub mod bounded;
mod channel;
pub mod receiver;
pub mod sender;
pub mod unbounded;
