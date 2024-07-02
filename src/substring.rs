//! The `substring` module provides the Substring trait that allows for extracting a substring
//! from a string.

pub trait Substring {
    /// Extracts a substring from the string.
    ///
    /// This method does not attempt to return graphemes only straight Unicode codes.
    ///
    /// # Arguments
    ///
    /// * `start` - The starting index of the substring.
    /// * `end` - The ending index of the substring.
    ///
    /// # Returns
    ///
    /// A String with the first character the `start` index and the last character the `end - 1` index.
    fn substring(&self, start: usize, end: usize) -> String;
}

impl Substring for String {
    fn substring(&self, start: usize, end: usize) -> String {
        self.chars().skip(start).take(end - start).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substring() {
        let s = "Hello, world!".to_string();
        assert_eq!(s.substring(0, 5), "Hello");
        assert_eq!(s.substring(7, 12), "world");
    }
}
