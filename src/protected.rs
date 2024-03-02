//! The `protected` module provides a simple wrapper around `Arc<Mutex<T>>` to allow for safe
//! sharing of data between threads.

use std::sync::{Arc, Mutex, MutexGuard};

/// A simple wrapper around `Arc<Mutex<T>>` to allow for safe sharing of data between threads.
/// Note that the type protected by this wrapper must implement Clone.
#[derive(Debug, Clone)]
pub struct Protected<T> {
    /// The `Arc<Mutex<T>>` that holds the data.
    item: Arc<Mutex<T>>,
}

impl<T> Protected<T> {
    /// Create a new `Protected<T>` with the given item.
    ///
    /// # Arguments
    ///
    /// * `item` - The item to protect.
    ///
    /// # Returns
    ///
    /// A new `Protected<T>` containing the given item.
    pub fn new(item: T) -> Protected<T> {
        Protected {
            item: Arc::new(Mutex::new(item)),
        }
    }

    /// Lock the protected item for access.
    ///
    /// This function assumes that unwrap() of the `Mutex` after a
    /// lock operation will always succeed.
    ///
    /// # Returns
    ///
    /// A `MutexGuard<T>` that allows access to the protected item.
    pub fn lock(&self) -> MutexGuard<T> {
        self.item.lock().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ops::Deref;

    #[test]
    fn test_create() {
        let protected_int = Protected::new(32);
        assert_eq!(protected_int.lock().deref(), &32);
    }
}
