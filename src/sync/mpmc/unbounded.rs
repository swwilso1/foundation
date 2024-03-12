//! The `unbounded` module provides the [`unbounded_channel`] function for creating an unbounded
//! channel.

use crate::sync::mpmc::{channel::Channel, receiver::Receiver, sender::Sender};
use std::sync::{Arc, Mutex};

/// Create an unbounded message channel
///
/// # Returns
///
/// A tuple of [`Sender`] and [`Receiver`]
pub fn unbounded_channel<T: Clone>() -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Mutex::new(Channel::new()));
    let sender = Sender::new(channel.clone(), None);
    let receiver = Receiver::new(channel);
    (sender, receiver)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::mpmc::bounded::tests::test_driver;

    #[tokio::test]
    async fn test_channel() {
        let (sender, mut receiver) = unbounded_channel::<i32>();
        let mut receiver2 = sender.subscribe();

        sender.send(1).await.unwrap();
        sender.send(2).await.unwrap();

        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));

        assert_eq!(receiver2.recv().await, Some(1));
        assert_eq!(receiver2.recv().await, Some(2));
    }

    #[tokio::test]
    async fn test_with_threads() {
        let creator = Box::new(|_bound| unbounded_channel::<i32>());
        test_driver(None, 500, 4, creator.clone()).await;
        test_driver(None, 20, 8, creator.clone()).await;
        test_driver(None, 7, 2, creator).await
    }
}
