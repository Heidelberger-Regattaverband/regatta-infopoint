use std::fmt::{Display, Formatter, Result};

use crate::utils;

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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "?{}", self.command)
    }
}

/// A message to respond with the list of open heats.
#[derive(Debug)]
pub(crate) struct ResponseListOpenHeats {
    /// A list of open heats.
    pub(crate) heats: Vec<Heat>,
}

impl ResponseListOpenHeats {
    /// Create a new response from the given message.
    pub(crate) fn new(message: &str) -> Self {
        let mut instance = ResponseListOpenHeats { heats: Vec::new() };
        for line in message.lines() {
            let heat = Heat::new_from_str(line);
            instance.heats.push(heat);
        }
        instance
    }
}

/// A heat in a competition.
#[derive(Debug)]
pub(crate) struct Heat {
    // The heat identifier.
    pub(crate) id: u16,
    // The heat number.
    pub(crate) number: u16,
    // The heat status.
    status: u8,
    // The boats in the heat.
    pub(crate) boats: Option<Vec<Boat>>,
}

impl Heat {
    /// Create a new heat with the given id, number, and status.
    /// # Arguments
    /// * `id` - The heat identifier.
    /// * `number` - The heat number.
    /// * `status` - The heat status.
    fn new(id: u16, number: u16, status: u8) -> Self {
        Heat {
            id,
            number,
            status,
            boats: None,
        }
    }

    pub(crate) fn new_from_str(heat_str: &str) -> Self {
        let parts: Vec<&str> = heat_str.split_whitespace().collect();
        if parts.len() != 3 {
            panic!("Invalid heat format: {}", heat_str);
        }
        let number = parts[0].parse().unwrap();
        let id = parts[1].parse().unwrap();
        let status = parts[2].parse().unwrap();
        Heat::new(id, number, status)
    }
}

impl Display for Heat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(
            f,
            "Heat: id={}, number={}, status={}",
            self.id, self.number, self.status
        )
    }
}

/// A message to request the start list of a heat.
pub(crate) struct RequestStartList<'a> {
    command: &'a str,
    id: u16,
}

impl RequestStartList<'_> {
    /// Create a new request to get the start list of a heat.
    /// # Arguments
    /// * `id` - The heat identifier.
    pub(crate) fn new(id: u16) -> Self {
        RequestStartList {
            command: "STARTLIST",
            id,
        }
    }
}

impl Display for RequestStartList<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "?{} id={}", self.command, self.id)
    }
}

/// A message to respond with the start list of a heat.
pub(crate) struct ResponseStartList {
    /// A list of boats in the heat.
    pub(crate) boats: Vec<Boat>,
}

impl ResponseStartList {
    pub(crate) fn new(message: String) -> Self {
        let mut instance = ResponseStartList { boats: Vec::new() };

        for line in message.lines() {
            let parts: Vec<&str> = line.splitn(3, ' ').collect();
            if parts.len() == 3 {
                let bib = parts[0].parse().unwrap();
                let lane = parts[1].parse().unwrap();
                let club = parts[2].to_owned();
                let boat = Boat::new(lane, bib, club);
                instance.boats.push(boat);
            }
        }
        instance
    }
}

/// A boat in a heat.
#[derive(Debug)]
pub(crate) struct Boat {
    /// The lane number the boat is starting in.
    pub(crate) lane: u8,
    /// The bib of the boat.
    pub(crate) bib: u8,
    /// The club name of the boat.
    pub(crate) club: String,
}

impl Boat {
    /// Create a new boat with the given lane, bib, and club.
    /// # Arguments
    /// * `lane` - The lane number.
    /// * `bib` - The bib.
    /// * `club` - The club name.
    pub(crate) fn new(lane: u8, bib: u8, club: String) -> Boat {
        Boat {
            bib,
            lane,
            club: utils::unquote(&club),
        }
    }
}

impl Display for Boat {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        writeln!(f, "Boat: lane={}, bib={}, club={}", self.lane, self.bib, self.club)
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

    #[test]
    fn test_request_start_list() {
        let request = RequestStartList::new(1);
        assert_eq!(request.to_string(), "?STARTLIST id=1\n");
    }

    #[test]
    fn test_response_start_list() {
        let message =
            "1 1 'RV Neptun Konstanz'\n2 2 'RG Heidelberg'\n3 3 'Heidelberger RK'\n4 4 'Marbacher RV'".to_owned();
        let response = ResponseStartList::new(message);
        assert_eq!(response.boats.len(), 4);
        assert_eq!(response.boats[0].lane, 1);
        assert_eq!(response.boats[0].bib, 1);
        assert_eq!(response.boats[0].club, "RV Neptun Konstanz");
        assert_eq!(response.boats[1].lane, 2);
        assert_eq!(response.boats[1].bib, 2);
        assert_eq!(response.boats[1].club, "RG Heidelberg");
        assert_eq!(response.boats[2].lane, 3);
        assert_eq!(response.boats[2].bib, 3);
        assert_eq!(response.boats[2].club, "Heidelberger RK");
    }

    #[test]
    fn test_boat() {
        let boat = Boat::new(1, 1, "RV Neptun Konstanz".to_owned());
        assert_eq!(boat.lane, 1);
        assert_eq!(boat.bib, 1);
        assert_eq!(boat.club, "RV Neptun Konstanz");
    }

    #[test]
    fn test_display_boat() {
        let boat = Boat::new(1, 1, "RV Neptun Konstanz".to_owned());
        assert_eq!(boat.to_string(), "Boat: lane=1, bib=1, club=RV Neptun Konstanz\n");
    }
}
