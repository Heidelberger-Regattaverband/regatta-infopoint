use std::fmt::{Display, Result};

/// A message to request the list of open heats.
pub(crate) struct RequestOpenHeats<'a> {
    command: &'a str,
}

impl RequestOpenHeats<'_> {
    /// Create a new request for the list of open heats.
    pub(crate) fn new() -> Self {
        RequestOpenHeats { command: "OPEN" }
    }
}

impl Display for RequestOpenHeats<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        writeln!(f, "?{}", self.command)
    }
}
