//! The `defer` module provides the `Defer` object which allows the programmer to register code
//! to be called when the `Defer` object goes out of scope. This is useful for running cleanup
//! code when a function returns, regardless of whether the function returns normally or with an
//! error.

/// The `Defer` struct provides a way to run code when the `Defer` object goes out of scope.
/// The downside of Rust's memory management incurs a penalty for using this pattern. Any variables
/// captured by the closure must use a type that you can share between closures. That implies the
/// use of things like `Arc`, `RwLock`, `Mutex`, etc. to share the captured variables between
/// closures.
///
/// # Example
///
/// ```rust
/// use foundation::defer::Defer;
///
/// fn main() {
///    let x = std::sync::Arc::new(std::sync::RwLock::new(0));
///    let x_c = x.clone();
///    {
///      let _defer = Defer::new(move || *x_c.write().unwrap() = 1);
///    }
///    assert_eq!(*x.read().unwrap(), 1);
/// }
/// ```
///
/// In using `Defer`, the programmer must often explicitly write a drop() statement in order
/// to force the compiler to not optimize away the deferred object.
pub struct Defer {
    action: Box<dyn FnMut() + Send + Sync + 'static>,
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
        F: FnMut() + Send + Sync + 'static,
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

    #[test]
    fn test_defer_runs_on_explicit_drop() {
        let x = Arc::new(RwLock::new(0));
        let x_c = x.clone();
        let defer = Defer::new(move || *x_c.write().unwrap() = 1);
        // The action should not have run yet.
        assert_eq!(*x.read().unwrap(), 0);
        drop(defer);
        assert_eq!(*x.read().unwrap(), 1);
    }

    #[test]
    fn test_multiple_defers_run_in_lifo_order() {
        let order = Arc::new(RwLock::new(Vec::new()));
        {
            let order_c = order.clone();
            let _defer1 = Defer::new(move || order_c.write().unwrap().push(1));
            let order_c = order.clone();
            let _defer2 = Defer::new(move || order_c.write().unwrap().push(2));
            let order_c = order.clone();
            let _defer3 = Defer::new(move || order_c.write().unwrap().push(3));
        }
        // Defer objects drop in reverse declaration order.
        assert_eq!(*order.read().unwrap(), vec![3, 2, 1]);
    }

    #[test]
    fn test_defer_runs_on_early_return() {
        fn run(early: bool, flag: Arc<RwLock<bool>>) -> i32 {
            let flag_c = flag.clone();
            let _defer = Defer::new(move || *flag_c.write().unwrap() = true);
            if early {
                return 1;
            }
            2
        }

        let flag = Arc::new(RwLock::new(false));
        assert_eq!(run(true, flag.clone()), 1);
        assert!(*flag.read().unwrap());

        let flag = Arc::new(RwLock::new(false));
        assert_eq!(run(false, flag.clone()), 2);
        assert!(*flag.read().unwrap());
    }

    #[test]
    fn test_defer_runs_on_panic() {
        let x = Arc::new(RwLock::new(0));
        let x_c = x.clone();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _defer = Defer::new(move || *x_c.write().unwrap() = 1);
            panic!("boom");
        }));
        assert!(result.is_err());
        // The deferred action should still run during unwinding.
        assert_eq!(*x.read().unwrap(), 1);
    }

    #[test]
    fn test_defer_captures_multiple_variables() {
        let a = Arc::new(RwLock::new(0));
        let b = Arc::new(RwLock::new(0));
        let a_c = a.clone();
        let b_c = b.clone();
        {
            let _defer = Defer::new(move || {
                *a_c.write().unwrap() = 10;
                *b_c.write().unwrap() = 20;
            });
        }
        assert_eq!(*a.read().unwrap(), 10);
        assert_eq!(*b.read().unwrap(), 20);
    }
}
