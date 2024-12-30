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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_list_open_heats() {
        let request = RequestListOpenHeats::new();
        assert_eq!(request.to_string(), "?OPEN\n");
    }

    #[test]
    fn test_response_list_open_heats() {
        let message = "3 2766 4\n50 2767 4\n71 2786 4";
        let response = ResponseListOpenHeats::new(message);
        assert_eq!(response.heats.len(), 3);
        assert_eq!(response.heats[0].id, 2766);
        assert_eq!(response.heats[0].number, 3);
        assert_eq!(response.heats[0].status, 4);
        assert_eq!(response.heats[1].id, 2767);
        assert_eq!(response.heats[1].number, 50);
        assert_eq!(response.heats[1].status, 4);
        assert_eq!(response.heats[2].id, 2786);
        assert_eq!(response.heats[2].number, 71);
        assert_eq!(response.heats[2].status, 4);
    }

    #[test]
    fn test_heat() {
        let heat = Heat::new(1, 1, 4);
        assert_eq!(heat.number, 1);
        assert_eq!(heat.id, 1);
        assert_eq!(heat.status, 4);
    }
}
