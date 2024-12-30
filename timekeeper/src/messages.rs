use std::fmt::{Display, Result};

/// A message to request the list of open heats.
pub(crate) struct RequestListOpenHeats<'a> {
    command: &'a str,
}

impl RequestListOpenHeats<'_> {
    /// Create a new request to get the list of open heats.
    pub(crate) fn new() -> Self {
        RequestListOpenHeats { command: "OPEN" }
    }
}

impl Display for RequestListOpenHeats<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result {
        writeln!(f, "?{}", self.command)
    }
}

/// A message to respond with the list of open heats.
pub(crate) struct ResponseListOpenHeats {
    /// A list of open heats.
    pub(crate) heats: Vec<Heat>,
}

impl ResponseListOpenHeats {
    /// Create a new response from the given message.
    pub(crate) fn new(message: &str) -> Self {
        let mut instance = ResponseListOpenHeats { heats: Vec::new() };
        for line in message.lines() {
            let parts: Vec<&str> = line.split(" ").collect();
            if parts.len() == 3 {
                let number = parts[0].parse().unwrap();
                let id = parts[1].parse().unwrap();
                let status = parts[2].parse().unwrap();
                let heat = Heat::new(id, number, status);
                instance.add_heat(heat);
            }
        }
        instance
    }

    fn add_heat(&mut self, heat: Heat) {
        self.heats.push(heat);
    }
}

pub(crate) struct Heat {
    id: u16,
    pub(crate) number: u16,
    status: u8,
}

impl Heat {
    fn new(id: u16, number: u16, status: u8) -> Self {
        Heat { id, number, status }
    }
}
