use crate::error::AquariusErr;
use crate::utils;
use ::chrono::{DateTime, Local};
use ::db::timekeeper::Split;
use ::serde::Serialize;
use ::std::fmt::{Display, Formatter, Result as FmtResult};
use ::std::str::FromStr;

pub(super) type Bib = u8;
type Lane = u8;
pub(super) type HeatNr = i16;

/// A message to request the list of open heats.
#[derive(Default)]
pub struct RequestListOpenHeats {}

impl Display for RequestListOpenHeats {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "?OPEN")
    }
}

/// A message to respond with the list of open heats.
#[derive(Debug)]
pub struct ResponseListOpenHeats {
    /// A list of open heats.
    pub(crate) heats: Vec<Heat>,
}

impl FromStr for ResponseListOpenHeats {
    type Err = AquariusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instance = ResponseListOpenHeats { heats: Vec::new() };
        for line in s.lines() {
            let heat = Heat::from_str(line)?;
            instance.heats.push(heat);
        }
        Ok(instance)
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

impl FromStr for ResponseStartList {
    type Err = AquariusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut instance = ResponseStartList { boats: Vec::new() };
        for line in s.lines() {
            let boat = Boat::from_str(line)?;
            instance.boats.push(boat);
        }
        Ok(instance)
    }
}

pub(super) struct RequestSetTime {
    pub(super) time: DateTime<Local>,
    pub(super) split: Split,
    pub(super) heat_nr: HeatNr,
    pub(super) bib: Option<Bib>,
}

impl Display for RequestSetTime {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let time = format!("{}", self.time.format("%H:%M:%S%.3f"));
        let split = u8::from(&self.split);
        match self.bib {
            Some(bib) => writeln!(f, "TIME time={time} comp={} split={split} bib={bib}", self.heat_nr),
            _ => writeln!(f, "TIME time={time} comp={} split={split}", self.heat_nr),
        }
    }
}

/// An event that a heat has changed. This event is sent when a heat is opened or closed
#[derive(Debug)]
pub struct EventHeatChanged {
    /// The heat that has changed.
    pub heat: Heat,
    /// Whether the heat has been opened or closed.
    pub opened: bool,
}

impl FromStr for EventHeatChanged {
    type Err = AquariusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 4 {
            return Err(AquariusErr::InvalidMessage(s.to_owned()));
        }

        let action = parts[0];
        let number = parts[1].parse()?;
        let id = parts[2].parse()?;
        let status = parts[3].parse()?;

        match action {
            "!OPEN+" => Ok(EventHeatChanged::new(Heat::new(id, number, status), true)),
            "!OPEN-" => Ok(EventHeatChanged::new(Heat::new(id, number, status), false)),
            _ => Err(AquariusErr::InvalidMessage(s.to_owned())),
        }
    }
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
}

/// A heat in a competition.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Heat {
    // The heat identifier.
    pub id: u16,
    // The heat number.
    pub number: HeatNr,
    // The heat state.
    state: u8,
    // The boats in the heat.
    pub boats: Option<Vec<Boat>>,
}

impl Heat {
    /// Create a new heat with the given id, number, and state.
    /// # Arguments
    /// * `id` - The heat identifier
    /// * `number` - The heat number.
    /// * `state` - The heat state.
    /// # Returns
    /// A new heat with the given id, number, and state.
    fn new(id: u16, number: i16, state: u8) -> Self {
        Heat {
            id,
            number,
            state,
            boats: None,
        }
    }
}

impl FromStr for Heat {
    type Err = AquariusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_whitespace().collect();
        if parts.len() != 3 {
            return Err(AquariusErr::InvalidMessage(s.to_owned()));
        }
        let number = parts[0].parse()?;
        let id = parts[1].parse()?;
        let status = parts[2].parse()?;
        Ok(Heat::new(id, number, status))
    }
}

impl Display for Heat {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        writeln!(f, "Heat: id={}, number={}, state={}", self.id, self.number, self.state)
    }
}

/// A boat in a heat.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Boat {
    /// The lane number the boat is starting in.
    pub lane: Lane,
    /// The bib of the boat.
    pub bib: Bib,
    /// The club name of the boat.
    pub club: String,
    /// The state of the boat.
    pub state: u8,
}

impl Boat {
    /// Create a new boat with the given lane, bib, and club.
    /// # Arguments
    /// * `lane` - The lane number.
    /// * `bib` - The bib.
    /// * `club` - The club name.
    /// # Returns
    /// A new boat.
    fn new(lane: Lane, bib: Bib, club: String, state: u8) -> Self {
        Boat {
            bib,
            lane,
            club: utils::unquote(&club).to_owned(),
            state,
        }
    }
}

impl FromStr for Boat {
    type Err = AquariusErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.splitn(4, ' ').collect();
        if parts.len() == 4 {
            let lane = parts[0].parse()?;
            let bib: u8 = parts[1].parse()?;
            let state: u8 = parts[2].parse()?;
            let club = parts[3].to_owned();
            Ok(Boat::new(lane, bib, club, state))
        } else {
            Err(AquariusErr::InvalidMessage(s.to_owned()))
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
    use tracing::Level;

    fn init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }

    #[test]
    fn test_request_list_open_heats() {
        let request = RequestListOpenHeats::default();
        assert_eq!(request.to_string(), "?OPEN\n");
    }

    #[test]
    fn test_response_list_open_heats() {
        let message = "3 2766 4\n50 2767 4\n71 2786 4";
        let response = ResponseListOpenHeats::from_str(message);
        assert!(response.is_ok());
        let response = response.unwrap();
        assert_eq!(response.heats.len(), 3);
        assert_eq!(response.heats[0].id, 2766);
        assert_eq!(response.heats[0].number, 3);
        assert_eq!(response.heats[0].state, 4);
        assert_eq!(response.heats[1].id, 2767);
        assert_eq!(response.heats[1].number, 50);
        assert_eq!(response.heats[1].state, 4);
        assert_eq!(response.heats[2].id, 2786);
        assert_eq!(response.heats[2].number, 71);
        assert_eq!(response.heats[2].state, 4);

        let message = "3 2766 4\n50 2767 4\n71 2786 4f";
        assert!(ResponseListOpenHeats::from_str(message).is_err());
    }

    #[test]
    fn test_heat_new() {
        let heat = Heat::new(1234, 50, 4);
        assert_eq!(heat.id, 1234);
        assert_eq!(heat.number, 50);
        assert_eq!(heat.state, 4);
    }

    #[test]
    fn test_heat_from_str() {
        init();

        assert!(Heat::from_str("50 1234 4 34").is_err());
        assert!(Heat::from_str("50f 1234 4").is_err());
        assert!(Heat::from_str("50 1234f 4").is_err());
        assert!(Heat::from_str("50 1234 4f").is_err());
    }

    #[test]
    fn test_display_heat() {
        let heat = Heat::new(2766, 1, 4);
        assert_eq!(heat.to_string(), "Heat: id=2766, number=1, state=4\n");
    }

    #[test]
    fn test_request_start_list() {
        let request = RequestStartList::new(1);
        assert_eq!(request.to_string(), "?STARTLIST id=1\n");
    }

    #[test]
    fn test_response_start_list() {
        let message =
            "1 1 0 'RV Neptun Konstanz'\n2 2 0 'RG Heidelberg'\n3 3 0 'Heidelberger RK'\n4 4 0 'Marbacher RV'"
                .to_owned();
        let response = ResponseStartList::from_str(&message);
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
        let boat = Boat::new(1, 12, "RV Neptun Konstanz".to_owned(), 0);
        assert_eq!(boat.lane, 1);
        assert_eq!(boat.bib, 12);
        assert_eq!(boat.club, "RV Neptun Konstanz");
    }

    #[test]
    fn test_boat_from_str() {
        let boat = Boat::from_str("1 12 0 'RV Neptun Konstanz'");
        assert!(boat.is_ok());
        let boat = boat.unwrap();
        assert_eq!(boat.lane, 1);
        assert_eq!(boat.bib, 12);
        assert_eq!(boat.club, "RV Neptun Konstanz");
        assert_eq!(boat.state, 0);

        assert!(Boat::from_str("1 12").is_err());
    }

    #[test]
    fn test_display_boat() {
        let boat = Boat::new(1, 12, "RV Neptun Konstanz".to_owned(), 0);
        assert_eq!(boat.to_string(), "Boat: lane=1, bib=12, club=RV Neptun Konstanz\n");
    }

    #[test]
    fn test_event_heat_changed_new() {
        let heat = Heat::new(1234, 50, 4);
        let event = EventHeatChanged::new(heat.clone(), true);
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.state, 4);
        assert!(event.opened);

        let event = EventHeatChanged::new(heat, false);
        assert!(!event.opened);
    }

    #[test]
    fn test_event_heat_changed_from_str() {
        init();

        let event = EventHeatChanged::from_str("!OPEN+ 50 1234 4");
        assert!(event.is_ok());
        let event = event.unwrap();
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.state, 4);
        assert!(event.opened);

        let event = EventHeatChanged::from_str("!OPEN- 50 1234 4");
        assert!(event.is_ok());
        let event = event.unwrap();
        assert_eq!(event.heat.id, 1234);
        assert_eq!(event.heat.number, 50);
        assert_eq!(event.heat.state, 4);
        assert!(!event.opened);

        let event = EventHeatChanged::from_str("!OPEN+ 50 1234");
        assert!(event.is_err());

        let event = EventHeatChanged::from_str("!OPEN+ 50 1234 4 34");
        assert!(event.is_err());

        let event = EventHeatChanged::from_str("!OPEN= 50 1234 4");
        assert!(event.is_err());
        let event = EventHeatChanged::from_str("!OPEN+ 50f 1234 4");
        assert!(event.is_err());
        let event = EventHeatChanged::from_str("!OPEN+ 50 1234f 4");
        assert!(event.is_err());
        let event = EventHeatChanged::from_str("!OPEN+ 50 1234 4f");
        assert!(event.is_err());
    }
}
