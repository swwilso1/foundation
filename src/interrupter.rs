/// The possible interruptions that can occur.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Interruption {
    /// The interruption is a stop.
    Stop,

    /// The interruption is a pause.
    Pause,

    /// The interruption is a resume.
    Resume,

    /// The interruption is an abort.
    Abort,
}

/// The `Interrupter` struct is used to manage interruptions.
#[derive(Debug, Clone)]
pub struct Interrupter {
    /// The current interruption state.
    interruption: Option<Interruption>,
}

impl Interrupter {
    /// Create a new `Interrupter` with no interruption.
    pub fn default() -> Interrupter {
        Interrupter { interruption: None }
    }

    /// Set the interruption state to the given interruption.
    ///
    /// # Arguments
    ///
    /// * `interruption` - The interruption for which to set the state.
    pub fn interrupt_with(&mut self, interruption: Interruption) {
        self.interruption = Some(interruption);
    }

    /// Check if the interruption is the given interruption.
    ///
    /// # Arguments
    ///
    /// * `interruption` - The interruption to check.
    ///
    /// # Returns
    ///
    /// `true` if the interruption is the given interruption, `false` otherwise.
    pub fn interrupt_is(&self, interruption: Interruption) -> bool {
        match &self.interruption {
            Some(int) => int == &interruption,
            None => false,
        }
    }

    /// Check if the something has interrupted.
    ///
    /// # Returns
    ///
    /// `true` if something has interrupted, `false` otherwise.
    pub fn interrupted(&self) -> bool {
        self.interruption.is_some()
    }

    /// Clear the interruption state.
    pub fn clear(&mut self) {
        self.interruption = None;
    }

    /// Get the current interruption state.
    pub fn get_interruption(&self) -> Option<Interruption> {
        self.interruption
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default() {
        let interrupter = Interrupter::default();
        assert_eq!(interrupter.interrupted(), false);
        assert_eq!(interrupter.get_interruption(), None);
    }

    #[test]
    fn test_interrupt_with() {
        let mut interrupter = Interrupter::default();
        interrupter.interrupt_with(Interruption::Pause);
        assert_eq!(interrupter.interrupted(), true);
        assert_eq!(interrupter.get_interruption(), Some(Interruption::Pause));
        assert_eq!(interrupter.interrupt_is(Interruption::Pause), true);
    }

    #[test]
    fn test_interrupt_is() {
        let mut interrupter = Interrupter::default();
        interrupter.interrupt_with(Interruption::Pause);
        assert_eq!(interrupter.interrupt_is(Interruption::Pause), true);
        assert_eq!(interrupter.interrupt_is(Interruption::Stop), false);
    }

    #[test]
    fn test_clear() {
        let mut interrupter = Interrupter::default();
        interrupter.interrupt_with(Interruption::Pause);
        interrupter.clear();
        assert_eq!(interrupter.interrupted(), false);
        assert_eq!(interrupter.get_interruption(), None);
    }
}
