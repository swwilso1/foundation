//! The `error` module provides error types used by channels in the sync module.

use std::error::Error;
use std::fmt;

/// Error returned by a `Sender`.
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SendError<T>(pub T);

impl<T> fmt::Debug for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SendError").finish_non_exhaustive()
    }
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "channel closed")
    }
}

impl<T> Error for SendError<T> {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_display() {
        // Display never reveals the contained value, only that the channel closed.
        let error = SendError(42);
        assert_eq!(format!("{}", error), "channel closed");
    }

    #[test]
    fn test_debug() {
        // Debug hides the contained value via finish_non_exhaustive.
        let error = SendError("secret");
        assert_eq!(format!("{:?}", error), "SendError { .. }");
    }

    #[test]
    fn test_is_std_error() {
        // SendError must be usable as a boxed std::error::Error.
        let error: Box<dyn Error> = Box::new(SendError(7));
        assert_eq!(error.to_string(), "channel closed");
    }

    #[test]
    fn test_equality_and_copy() {
        // PartialEq/Eq/Clone/Copy derives.
        let a = SendError(1);
        let b = SendError(1);
        let c = SendError(2);
        assert_eq!(a, b);
        assert_ne!(a, c);

        let copied = a;
        assert_eq!(a, copied);

        let cloned = a.clone();
        assert_eq!(a, cloned);
    }
}
