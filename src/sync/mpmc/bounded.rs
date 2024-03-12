//! The `bounded` module provides the [`channel`] function for creating a bounded channel.

use crate::sync::mpmc::{channel::Channel, receiver::Receiver, sender::Sender};
use std::sync::{Arc, Mutex};

/// Create a bounded message channel
///
/// # Arguments
///
/// * `bound` - The number of messages allowed in the channel
///
/// # Returns
///
/// A tuple of [`Sender`] and [`Receiver`]
pub fn channel<T: Clone>(bound: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Mutex::new(Channel::new()));
    let sender = Sender::new(channel.clone(), Some(bound));
    let receiver = Receiver::new(channel);
    (sender, receiver)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tokio::task::JoinHandle;

    #[tokio::test]
    async fn test_channel() {
        let (sender, mut receiver) = channel::<i32>(2);
        let mut receiver2 = sender.subscribe();

        sender.send(1).await.unwrap();
        sender.send(2).await.unwrap();

        assert_eq!(receiver.recv().await, Some(1));
        assert_eq!(receiver.recv().await, Some(2));

        assert_eq!(receiver2.recv().await, Some(1));
        assert_eq!(receiver2.recv().await, Some(2));
    }

    pub(crate) async fn thread_test_helper<
        T: Clone + std::fmt::Display + PartialEq + std::fmt::Debug,
    >(
        mut receiver: Receiver<T>,
        first_received: T,
        last_received: T,
        _number: i32,
    ) {
        let mut last_thing = first_received;
        loop {
            match receiver.recv().await {
                Some(thing) => {
                    last_thing = thing;
                }
                None => {
                    assert_eq!(last_thing, last_received);
                    break;
                }
            }
        }
    }

    pub(crate) async fn test_driver(
        bound: Option<usize>,
        max_send: i32,
        mut receivers: usize,
        channel_creator: Box<dyn Fn(Option<usize>) -> (Sender<i32>, Receiver<i32>)>,
    ) {
        let (sender, receiver) = channel_creator(bound);
        let mut handles: Vec<JoinHandle<()>> = Vec::new();
        let mut receiver_number: i32 = 1;

        while receivers > 1 {
            let receiver = sender.subscribe();
            let h = tokio::spawn(async move {
                thread_test_helper(receiver, max_send, 1, receiver_number).await
            });
            handles.push(h);
            receivers -= 1;
            receiver_number += 1;
        }

        let h = tokio::spawn(async move {
            thread_test_helper(receiver, max_send, 1, receiver_number).await
        });

        handles.push(h);

        let s = tokio::spawn(async move {
            let mut i = max_send;
            while i > 0 {
                sender.send(i).await.unwrap();
                i -= 1;
            }
        });

        handles.push(s);

        futures::future::join_all(handles).await;
    }

    #[tokio::test]
    async fn test_with_threads() {
        let creator = Box::new(|bound| {
            if let Some(bound) = bound {
                channel::<i32>(bound)
            } else {
                panic!("Invalid bound value for bounded channel test");
            }
        });
        test_driver(Some(2), 500, 4, creator.clone()).await;
        test_driver(Some(3), 20, 8, creator.clone()).await;
        test_driver(Some(5), 7, 2, creator).await
    }

    #[tokio::test]
    async fn test_bounded_sends() {
        let (sender, mut receiver) = channel(2);
        let mut receiver2 = sender.subscribe();

        tokio::spawn(async move {
            assert_eq!(receiver.recv().await.unwrap(), 10);
            assert_eq!(receiver.recv().await.unwrap(), 11);
            assert_eq!(receiver.recv().await.unwrap(), 12);
        });

        tokio::spawn(async move {
            assert_eq!(receiver2.recv().await.unwrap(), 10);
            assert_eq!(receiver2.recv().await.unwrap(), 11);
            assert_eq!(receiver2.recv().await.unwrap(), 12);
        });

        sender.send(10).await.unwrap();
        sender.send(11).await.unwrap();
        sender.send(12).await.unwrap();
    }
}
