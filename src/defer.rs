//! The `defer` module provides the `Defer` object which allows the programmer to register code
//! to be called when the `Defer` object goes out of scope. This is useful for running cleanup
//! code when a function returns, regardless of whether the function returns normally or with an
//! error.

/// The `Defer` struct provides a way to run code when the `Defer` object goes out of scope.
/// The downside of Rust's memory management incurs a penalty for using this pattern. Any variables
/// captured by the closure must use a type that you can share between closures. That implies the
/// use of things like `Arc`, `RwLock`, `Mutex`, etc. to share the captured variables between
/// closures.
pub struct Defer {
    action: Box<dyn FnMut() -> () + Send + Sync + 'static>,
}

impl Defer {
    /// The `new` function creates a new `Defer` object with the given action.
    ///
    /// # Arguments
    ///
    /// * `action` - The action to run when the `Defer` object goes out of scope.
    ///
    /// # Returns
    ///
    /// A new `Defer` object with the given action.
    pub fn new<F>(action: F) -> Defer
    where
        F: FnMut() -> () + Send + Sync + 'static,
    {
        Defer {
            action: Box::new(action),
        }
    }
}

impl Drop for Defer {
    fn drop(&mut self) {
        (self.action)();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, RwLock};

    #[test]
    fn test_defer() {
        let x = Arc::new(RwLock::new(0));
        let x_c = x.clone();
        {
           let _defer = Defer::new(move || *x_c.write().unwrap() = 1);
        }
        assert_eq!(*x.read().unwrap(), 1);
    }
}