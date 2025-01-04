use crate::{error::MessageErr, utils};
use log::warn;
use std::fmt::{Display, Formatter, Result as FmtResult};

/// A message to request the list of open heats.
pub(crate) struct RequestListOpenHeats {}

impl RequestListOpenHeats {
    /// Create a new request to get the list of open heats.
    pub(crate) fn new() -> Self {
        RequestListOpenHeats {}
    }
}

impl Display for RequestListOpenHeats {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "?OPEN")
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
    /// # Arguments
    /// * `message` - The message to parse.
    /// # Returns
    /// The parsed response or `None` if the message is invalid.
    pub(crate) fn parse(message: &str) -> Option<Self> {
        let mut instance = ResponseListOpenHeats { heats: Vec::new() };
        for line in message.lines() {
            if let Some(heat) = Heat::parse(line) {
                instance.heats.push(heat);
            } else {
                warn!("Invalid heat format: {}", line);
                return None;
            }
        }
        Some(instance)
    }
}

/// A heat in a competition.
#[derive(Debug, Clone)]
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
    /// * `id` - The heat identifier
    /// * `number` - The heat number.
    /// * `status` - The heat status.
    /// # Returns
    /// A new heat with the given id, number, and status.
    fn new(id: u16, number: u16, status: u8) -> Self {
        Heat {
            id,
            number,
            status,
            boats: None,
        }
    }

    /// Parse a heat from a string.
    /// # Arguments
    /// * `heat_str` - The string to parse.
    /// # Returns
    /// The parsed heat or `None` if the string is invalid.
    pub(crate) fn parse(heat_str: &str) -> Option<Self> {
        let parts: Vec<&str> = heat_str.split_whitespace().collect();
        if parts.len() != 3 {
            warn!("Invalid heat format: {}", heat_str);
            return None;
        }
        let number = match parts[0].parse() {
            Ok(number) => number,
            Err(_) => {
                warn!("Invalid heat number: {}", parts[0]);
                return None;
            }
        };
        let id = match parts[1].parse() {
            Ok(id) => id,
            Err(_) => {
                warn!("Invalid heat ID: {}", parts[1]);
                return None;
            }
        };
        let status = match parts[2].parse() {
            Ok(status) => status,
            Err(_) => {
                warn!("Invalid status: {}", parts[2]);
                return None;
            }
        };
        Some(Heat::new(id, number, status))
    }
}

impl Display for Heat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(
            f,
            "Heat: id={}, number={}, status={}",
            self.id, self.number, self.status
        )
    }
}

/// A message to request the start list of a heat.
pub(crate) struct RequestStartList {
    id: u16,
}

impl RequestStartList {
    /// Create a new request to get the start list of a heat.
    /// # Arguments
    /// * `id` - The heat identifier.
    /// # Returns
    /// A new request to get the start list of a heat.
    pub(crate) fn new(id: u16) -> Self {
        RequestStartList { id }
    }
}

impl Display for RequestStartList {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "?STARTLIST id={}", self.id)
    }
}

/// A message to respond with the start list of a heat.
#[derive(Debug)]
pub(crate) struct ResponseStartList {
    /// A list of boats in the heat.
    pub(crate) boats: Vec<Boat>,
}

impl ResponseStartList {
    /// Parse the response from a string.
    /// # Arguments
    /// * `message` - The message to parse.
    /// # Returns
    /// The parsed response or an error if the message is invalid.
    pub(crate) fn parse(message: String) -> Result<Self, MessageErr> {
        let mut instance = ResponseStartList { boats: Vec::new() };

        for line in message.lines() {
            let boat = Boat::parse(line)?;
            instance.boats.push(boat);
        }
        Ok(instance)
    }
}

/// An event that a heat has changed. This event is sent when a heat is opened or closed
pub(crate) struct EventHeatChanged {
    /// The heat that has changed.
    pub(crate) heat: Heat,
    /// Whether the heat has been opened or closed.
    pub(crate) opened: bool,
}

impl EventHeatChanged {
    /// Create a new event that a heat has changed.
    /// # Arguments
    /// * `heat` - The heat that has changed.
    /// * `opened` - Whether the heat has been opened or closed.
    /// # Returns
    /// A new event that a heat has changed.
    pub(crate) fn new(heat: Heat, opened: bool) -> Self {
        EventHeatChanged { heat, opened }
    }

    /// Parse an event from a string.
    /// # Arguments
    /// * `event_str` - The string to parse.
    /// # Returns
    /// The parsed event.
    pub(crate) fn parse(event_str: &str) -> Option<Self> {
        let parts: Vec<&str> = event_str.split_whitespace().collect();
        if parts.len() != 4 {
            warn!("Invalid event format: {}", event_str);
            return None;
        }

        let action = parts[0];
        let number = match parts[1].parse() {
            Ok(number) => number,
            Err(_) => {
                warn!("Invalid heat number: {}", parts[1]);
                return None;
            }
        };
        let id = match parts[2].parse() {
            Ok(id) => id,
            Err(_) => {
                warn!("Invalid heat ID: {}", parts[2]);
                return None;
            }
        };
        let status = match parts[3].parse() {
            Ok(status) => status,
            Err(_) => {
                warn!("Invalid status: {}", parts[3]);
                return None;
            }
        };

        match action {
            "!OPEN+" => Some(EventHeatChanged::new(Heat::new(id, number, status), true)),
            "!OPEN-" => Some(EventHeatChanged::new(Heat::new(id, number, status), false)),
            _ => None,
        }
    }
}

/// A boat in a heat.
#[derive(Debug, Clone)]
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
    /// # Returns
    /// A new boat.
    fn new(lane: u8, bib: u8, club: String) -> Self {
        Boat {
            bib,
            lane,
            club: utils::unquote(&club),
        }
    }

    /// Parse a boat from a string in the format "lane bib club".
    ///
    /// # Arguments
    /// * `boat_str` - The string to parse.
    /// # Returns
    /// The parsed boat or an error if the string is invalid.
    pub(crate) fn parse(boat_str: &str) -> Result<Self, MessageErr> {
        let parts: Vec<&str> = boat_str.splitn(3, ' ').collect();
        if parts.len() == 3 {
            let lane = parts[0].parse().map_err(MessageErr::ParseError)?;
            let bib = parts[1].parse().map_err(MessageErr::ParseError)?;
            let club = parts[2].to_owned();
            Ok(Boat::new(lane, bib, club))
        } else {
            Err(MessageErr::InvalidMessage)
        }
    }
}

impl Display for Boat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Boat: lane={}, bib={}, club={}", self.lane, self.bib, self.club)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;

    #[test]
    fn test_request_list_open_heats() {
        let request = RequestListOpenHeats::new();
        assert_eq!(request.to_string(), "?OPEN\n");
    }

    #[test]
    fn test_response_list_open_heats() {
        let message = "3 2766 4\n50 2767 4\n71 2786 4";
        let response_opt = ResponseListOpenHeats::parse(message);
        assert!(response_opt.is_some());
        let response = response_opt.unwrap();
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

        let message = "3 2766 4\n50 2767 4\n71 2786 4f";
        assert!(ResponseListOpenHeats::parse(message).is_none());
    }

    #[test]
    fn test_heat_new() {
        let heat = Heat::new(1234, 50, 4);
        assert_eq!(heat.id, 1234);
        assert_eq!(heat.number, 50);
        assert_eq!(heat.status, 4);
    }

    #[test]
    fn test_heat_parse() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();
        assert!(Heat::parse("50 1234 4 34").is_none());
        assert!(Heat::parse("50f 1234 4").is_none());
        assert!(Heat::parse("50 1234f 4").is_none());
        assert!(Heat::parse("50 1234 4f").is_none());
    }

    #[test]
    fn test_display_heat() {
        let heat = Heat::new(2766, 1, 4);
        assert_eq!(heat.to_string(), "Heat: id=2766, number=1, status=4\n");
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
        let response = ResponseStartList::parse(message);
        assert!(response.is_ok());
        let response = response.unwrap();
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
    fn test_boat_new() {
        let boat = Boat::new(1, 12, "RV Neptun Konstanz".to_owned());
        assert_eq!(boat.lane, 1);
        assert_eq!(boat.bib, 12);
        assert_eq!(boat.club, "RV Neptun Konstanz");
    }

    #[test]
    fn test_boat_parse() {
        let boat = Boat::parse("1 12 'RV Neptun Konstanz'");
        assert!(boat.is_ok());
        let boat = boat.unwrap();
        assert_eq!(boat.lane, 1);
        assert_eq!(boat.bib, 12);
        assert_eq!(boat.club, "RV Neptun Konstanz");

        assert!(Boat::parse("1 12").is_err());
    }

    #[test]
    fn test_display_boat() {
        let boat = Boat::new(1, 12, "RV Neptun Konstanz".to_owned());
        assert_eq!(boat.to_string(), "Boat: lane=1, bib=12, club=RV Neptun Konstanz\n");
    }

    #[test]
    fn test_event_heat_changed() {
        let heat = Heat::new(1234, 50, 4);
        let event = EventHeatChanged::new(heat.clone(), true);
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.status, 4);
        assert!(event.opened);

        let event = EventHeatChanged::new(heat, false);
        assert!(!event.opened);
    }

    #[test]
    fn test_event_heat_changed_parse() {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(LevelFilter::Trace)
            .try_init();

        let event = EventHeatChanged::parse("!OPEN+ 50 1234 4");
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.status, 4);
        assert!(event.opened);

        let event = EventHeatChanged::parse("!OPEN- 50 1234 4");
        assert!(event.is_some());
        let event = event.unwrap();
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.status, 4);
        assert!(!event.opened);

        let event = EventHeatChanged::parse("!OPEN+ 50 1234");
        assert!(event.is_none());

        let event = EventHeatChanged::parse("!OPEN+ 50 1234 4 34");
        assert!(event.is_none());

        let event = EventHeatChanged::parse("!OPEN= 50 1234 4");
        assert!(event.is_none());
        let event = EventHeatChanged::parse("!OPEN+ 50f 1234 4");
        assert!(event.is_none());
        let event = EventHeatChanged::parse("!OPEN+ 50 1234f 4");
        assert!(event.is_none());
        let event = EventHeatChanged::parse("!OPEN+ 50 1234 4f");
        assert!(event.is_none());
    }
}
