use crate::db::utils::Column;
use serde::Serialize;
use std::time::Duration;
use tiberius::{time::chrono::NaiveDateTime, Query, Row};

#[derive(Debug, Serialize, Clone)]
pub struct Regatta {
    pub id: i32,
    title: String,
    sub_title: String,
    venue: String,
    #[serde(rename = "startDate")]
    start_date: String,
    #[serde(rename = "endDate")]
    end_date: String,
}
impl Regatta {
    pub fn from(row: &Row) -> Self {
        let start_date: NaiveDateTime = Column::get(row, "Event_StartDate");
        let end_date: NaiveDateTime = Column::get(row, "Event_EndDate");

        Regatta {
            id: Column::get(row, "Event_ID"),
            title: Column::get(row, "Event_Title"),
            sub_title: Column::get(row, "Event_SubTitle"),
            venue: Column::get(row, "Event_Venue"),
            start_date: start_date.date().to_string(),
            end_date: end_date.date().to_string(),
        }
    }

    pub(super) fn query_all<'a>() -> Query<'a> {
        Query::new("SELECT * FROM Event e")
    }

    pub(super) fn query_single<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT * FROM Event e WHERE e.Event_ID = @P1");
        query.bind(regatta_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct AgeClass {
    id: i32,
    caption: String,
    abbreviation: String,
    suffix: String,
    gender: String,
    #[serde(rename = "numSubClasses")]
    num_sub_classes: u8,
}
impl AgeClass {
    pub fn from(row: &Row) -> Self {
        let id = Column::get(row, "AgeClass_ID");
        let caption = Column::get(row, "AgeClass_Caption");
        let abbreviation = Column::get(row, "AgeClass_Abbr");
        let suffix = Column::get(row, "AgeClass_Suffix");
        let gender = Column::get(row, "AgeClass_Gender");
        let num_sub_classes = Column::get(row, "AgeClass_NumSubClasses");
        AgeClass {
            id,
            caption,
            abbreviation,
            suffix,
            gender,
            num_sub_classes,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct BoatClass {
    id: i32,
    caption: String,
    abbreviation: String,
    #[serde(rename = "numRowers")]
    num_rowers: i32,
    coxed: bool,
}
impl BoatClass {
    pub fn from(row: &Row) -> Self {
        let id = Column::get(row, "BoatClass_ID");
        let caption = Column::get(row, "BoatClass_Caption");
        let abbreviation = Column::get(row, "BoatClass_Abbr");
        let num_rowers = Column::get(row, "BoatClass_NumRowers");
        let coxed: u8 = Column::get(row, "BoatClass_Coxed");
        BoatClass {
            id,
            caption,
            abbreviation,
            num_rowers,
            coxed: coxed > 0,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Race {
    pub id: i32,
    number: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
    #[serde(rename = "longLabel")]
    long_label: String,
    comment: String,
    distance: i16,
    lightweight: bool,
    #[serde(rename = "raceMode")]
    race_mode: String,
    cancelled: bool,
    #[serde(rename = "registrationsCount")]
    registrations_count: i32,
    seeded: bool,
    #[serde(rename = "ageClass")]
    age_class: AgeClass,
    #[serde(rename = "boatClass")]
    boat_class: BoatClass,
}
impl Race {
    pub fn from(row: &Row) -> Self {
        let short_label: String = Column::get(row, "Offer_ShortLabel");
        let long_label: String = Column::get(row, "Offer_LongLabel");
        let comment: String = Column::get(row, "Offer_Comment");

        Race {
            id: Column::get(row, "Offer_ID"),
            comment: comment.trim().to_owned(),
            number: Column::get(row, "Offer_RaceNumber"),
            short_label: short_label.trim().to_owned(),
            long_label: long_label.trim().to_owned(),
            distance: Column::get(row, "Offer_Distance"),
            lightweight: Column::get(row, "Offer_IsLightweight"),
            race_mode: Column::get(row, "RaceMode_Title"),
            cancelled: Column::get(row, "Offer_Cancelled"),
            registrations_count: Column::get(row, "Registrations_Count"),
            seeded: Column::get(row, "isSet"),
            age_class: AgeClass::from(row),
            boat_class: BoatClass::from(row),
        }
    }

    pub(super) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT o.*, ac.*, bc.*, rm.*, hrv_o.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            JOIN RaceMode AS rm ON o.Offer_RaceMode_ID_FK = rm.RaceMode_ID
            JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            WHERE o.Offer_Event_ID_FK = @P1 ORDER BY o.Offer_SortValue ASC");
        query.bind(regatta_id);
        query
    }

    pub(super) fn query_single<'a>(race_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT o.*, rm.*,
            (SELECT Count(*) FROM Entry e WHERE e.Entry_Race_ID_FK = o.Offer_ID AND e.Entry_CancelValue = 0) as Registrations_Count
            FROM Offer o
            JOIN RaceMode AS rm ON o.Offer_RaceMode_ID_FK = rm.RaceMode_ID
            WHERE o.Offer_ID = @P1");
        query.bind(race_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Athlete {
    id: i32,
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
    gender: String,
    year: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Crew {
    id: i32,
    pos: u8,
    cox: bool,
    athlete: Athlete,
}
impl Crew {
    pub fn from(row: &Row) -> Crew {
        let dob: NaiveDateTime = Column::get(row, "Athlet_DOB");

        Crew {
            id: Column::get(row, "Crew_ID"),
            pos: Column::get(row, "Crew_Pos"),
            cox: Column::get(row, "Crew_IsCox"),
            athlete: Athlete {
                id: Column::get(row, "Athlet_ID"),
                first_name: Column::get(row, "Athlet_FirstName"),
                last_name: Column::get(row, "Athlet_LastName"),
                gender: Column::get(row, "Athlet_Gender"),
                year: dob.date().format("%Y").to_string(),
            },
        }
    }
    pub fn query_all<'a>(registration_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT DISTINCT c.Crew_ID, c.Crew_Pos, c.Crew_IsCox, a.Athlet_ID, a.Athlet_FirstName, a.Athlet_LastName, a.Athlet_Gender, Athlet_DOB
            FROM Crew c
            JOIN Athlet AS a ON c.Crew_Athlete_ID_FK = a.Athlet_ID
            WHERE c.Crew_Entry_ID_FK = @P1 AND c.Crew_RoundFrom <= 64 AND 64 <= c.Crew_RoundTo
            ORDER BY c.Crew_pos ASC",
        );
        query.bind(registration_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Registration {
    pub(crate) id: i32,
    bib: i16,
    #[serde(rename = "boatNumber")]
    boat_number: i16,
    comment: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
    club: Club,
    cancelled: bool,
    pub(crate) crew: Option<Vec<Crew>>,
}
impl Registration {
    pub fn from(row: &Row) -> Registration {
        let cancel_value: u8 = Column::get(row, "Entry_CancelValue");
        let cancelled = cancel_value > 0;
        let id = Column::get(row, "Entry_ID");
        Crew::query_all(id);
        Registration {
            id,
            bib: Column::get(row, "Entry_Bib"),
            comment: Column::get(row, "Entry_Comment"),
            boat_number: Column::get(row, "Entry_BoatNumber"),
            short_label: Column::get(row, "Label_Short"),
            cancelled,
            club: Club::from(row),
            crew: Option::None,
        }
    }

    pub fn query_all<'a>(race_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT DISTINCT e.*, l.Label_Short, c.Club_ID, c.Club_Abbr, c.Club_City
            FROM Entry e
            JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
            JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
            JOIN Club AS c ON c.Club_ID = e.Entry_OwnerClub_ID_FK
            WHERE e.Entry_Race_ID_FK = @P1 AND el.EL_RoundFrom <= 64 AND 64 <= el.EL_RoundTo
            ORDER BY e.Entry_Bib ASC",
        );
        query.bind(race_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Heat {
    pub id: i32,
    number: i16,
    round_code: String,
    label: String,
    group_value: i16,
    state: u8,
    cancelled: bool,
    date: String,
    time: String,
    race: Race,
    referee: Referee,
}
impl Heat {
    pub fn from(row: &Row) -> Self {
        let date_time: NaiveDateTime = Column::get(row, "Comp_DateTime");

        Heat {
            id: Column::get(row, "Comp_ID"),
            race: Race::from(row),
            number: Column::get(row, "Comp_Number"),
            round_code: Column::get(row, "Comp_RoundCode"),
            label: Column::get(row, "Comp_Label"),
            group_value: Column::get(row, "Comp_GroupValue"),
            state: Column::get(row, "Comp_State"),
            cancelled: Column::get(row, "Comp_Cancelled"),
            date: date_time.date().to_string(),
            time: date_time.time().to_string(),
            referee: Referee::from(row),
        }
    }

    pub(super) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT DISTINCT c.*, ac.*, bc.*, r.*, rm.RaceMode_Title, hrv_o.*,
            o.Offer_RaceNumber, o.Offer_ID, o.Offer_ShortLabel, o.Offer_LongLabel, o.Offer_Comment, o.Offer_Distance, o.Offer_IsLightweight, o.Offer_Cancelled
            FROM Comp AS c
            FULL OUTER JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
            JOIN RaceMode AS rm ON o.Offer_RaceMode_ID_FK = rm.RaceMode_ID
            FULL OUTER JOIN HRV_Offer AS hrv_o ON o.Offer_ID = hrv_o.id
            FULL OUTER JOIN AgeClass AS ac ON o.Offer_AgeClass_ID_FK = ac.AgeClass_ID
            JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
            FULL OUTER JOIN CompReferee AS cr ON cr.CompReferee_Comp_ID_FK = c.Comp_ID
            FULL OUTER JOIN Referee AS r ON r.Referee_ID = cr.CompReferee_Referee_ID_FK
            WHERE c.Comp_Event_ID_FK = @P1 ORDER BY c.Comp_DateTime ASC");
        query.bind(regatta_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HeatRegistration {
    pub id: i32,
    lane: i16,
    result: HeatResult,
    registration: Registration,
}
impl HeatRegistration {
    pub fn from(row: &Row) -> Self {
        HeatRegistration {
            id: Column::get(row, "CE_ID"),
            lane: Column::get(row, "CE_Lane"),
            registration: Registration::from(row),
            result: HeatResult::from(row),
        }
    }

    pub(super) fn query_all<'a>(heat_id: i32) -> Query<'a> {
        let mut query = Query::new("SELECT	DISTINCT ce.*, e.Entry_Bib, e.Entry_ID, e.Entry_BoatNumber, e.Entry_Comment, e.Entry_CancelValue, l.Label_Short, r.Result_Rank, r.Result_DisplayValue, r.Result_Delta, bc.BoatClass_NumRowers, cl.Club_ID, cl.Club_Abbr, cl.Club_City
          FROM CompEntries AS ce
          JOIN Comp AS c ON ce.CE_Comp_ID_FK = c.Comp_ID
          JOIN Offer AS o ON o.Offer_ID = c.Comp_Race_ID_FK
          JOIN BoatClass AS bc ON o.Offer_BoatClass_ID_FK = bc.BoatClass_ID
          FULL OUTER JOIN Entry AS e ON ce.CE_Entry_ID_FK = e.Entry_ID
          FULL OUTER JOIN EntryLabel AS el ON el.EL_Entry_ID_FK = e.Entry_ID
          FULL OUTER JOIN Label AS l ON el.EL_Label_ID_FK = l.Label_ID
          FULL OUTER JOIN Result AS r ON r.Result_CE_ID_FK = ce.CE_ID
          JOIN Club AS cl ON cl.Club_ID = e.Entry_OwnerClub_ID_FK
          WHERE ce.CE_Comp_ID_FK = @P1 AND (r.Result_SplitNr = 64 OR r.Result_SplitNr IS NULL)
            AND el.EL_RoundFrom <= c.Comp_Round AND c.Comp_Round <= el.EL_RoundTo");
        query.bind(heat_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct HeatResult {
    #[serde(rename = "rankSort")]
    rank_sort: u8,
    #[serde(rename = "rankLabel")]
    rank_label: String,
    result: String,
    delta: String,
    points: u8,
}
impl HeatResult {
    fn from(row: &Row) -> Self {
        let rank: u8 = Column::get(row, "Result_Rank");
        let rank_sort: u8 = if rank == 0 { u8::MAX } else { rank };
        let delta: String = if rank > 0 {
            let delta: i32 = Column::get(row, "Result_Delta");
            let duration = Duration::from_millis(delta as u64);
            let seconds = duration.as_secs();
            let millis = duration.subsec_millis() / 10;
            format!("{}.{}", seconds, millis)
        } else {
            Default::default()
        };

        let rank_label: String = if rank == 0 {
            Default::default()
        } else {
            rank.to_string()
        };

        let num_rowers: u8 = Column::get(row, "BoatClass_NumRowers");
        let points: u8 = if rank > 0 { num_rowers + (5 - rank) } else { 0 };

        HeatResult {
            delta,
            rank_label,
            rank_sort,
            result: Column::get(row, "Result_DisplayValue"),
            points,
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
pub struct Referee {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
}
impl Referee {
    fn from(row: &Row) -> Self {
        let last_name: String = Column::get(row, "Referee_LastName");
        let first_name: String = Column::get(row, "Referee_FirstName");
        if last_name.is_empty() && first_name.is_empty() {
            return Default::default();
        }
        Referee {
            last_name,
            first_name,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Club {
    id: i32,
    #[serde(rename = "shortName")]
    short_name: String,
    city: String,
}
impl Club {
    pub fn from(row: &Row) -> Self {
        Club {
            id: Column::get(row, "Club_ID"),
            short_name: Column::get(row, "Club_Abbr"),
            city: Column::get(row, "Club_City"),
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Score {
    rank: i16,
    points: f64,
    club: Club,
}
impl Score {
    pub(super) fn from(row: &Row) -> Self {
        Score {
            rank: Column::get(row, "rank"),
            points: Column::get(row, "points"),
            club: Club::from(row),
        }
    }

    pub(super) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT s.rank, s.points, c.Club_Name, c.Club_Abbr, c.Club_City
              FROM HRV_Score s
              JOIN Club AS c ON s.club_id = c.Club_ID
              WHERE s.event_id = @P1 ORDER BY s.rank ASC",
        );
        query.bind(regatta_id);
        query
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Statistics {
    races: RacesStatistics,
    heats: HeatsStatistics,
    registrations: RegistrationsStatistics,
}

#[derive(Debug, Serialize, Clone)]
struct RacesStatistics {
    all: i32,
    cancelled: i32,
}

#[derive(Debug, Serialize, Clone)]
struct HeatsStatistics {
    all: i32,
    cancelled: i32,
    pending: i32,
    started: i32,
    finished: i32,
    official: i32,
}

#[derive(Debug, Serialize, Clone)]
struct RegistrationsStatistics {
    all: i32,
    cancelled: i32,
    #[serde(rename = "registeringClubs")]
    registering_clubs: i32,
    athletes: i32,
    clubs: i32,
}

impl Statistics {
    pub(super) fn from(row: &Row) -> Self {
        let races = RacesStatistics {
            all: Column::get(row, "races_all"),
            cancelled: Column::get(row, "races_cancelled"),
        };
        let heats = HeatsStatistics {
            all: Column::get(row, "heats_all"),
            cancelled: Column::get(row, "heats_cancelled"),
            finished: Column::get(row, "heats_finished"),
            official: Column::get(row, "heats_official"),
            pending: Column::get(row, "heats_pending"),
            started: Column::get(row, "heats_started"),
        };
        let registrations = RegistrationsStatistics {
            all: Column::get(row, "registrations_all"),
            cancelled: Column::get(row, "registrations_cancelled"),
            registering_clubs: Column::get(row, "registrations_owner_clubs"),
            athletes: Column::get(row, "registrations_athletes"),
            clubs: Column::get(row, "registrations_clubs"),
        };
        Statistics {
            races,
            heats,
            registrations,
        }
    }

    pub(super) fn query_all<'a>(regatta_id: i32) -> Query<'a> {
        let mut query = Query::new(
            "SELECT
            (SELECT COUNT(*) FROM Offer o WHERE o.Offer_Event_ID_FK = @P1) AS races_all,
            (SELECT COUNT(*) FROM Offer o WHERE o.Offer_Event_ID_FK = @P1 AND o.Offer_Cancelled > 0) AS races_cancelled,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 ) AS heats_all,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_Cancelled > 0 ) AS heats_cancelled,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 4 ) AS heats_official,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 5 OR c.Comp_State = 6 ) AS heats_finished,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State = 2 ) AS heats_started,
            (SELECT COUNT(*) FROM Comp c WHERE c.Comp_Event_ID_FK = @P1 AND c.Comp_State < 2 AND c.Comp_Cancelled = 0 ) AS heats_pending,
            (SELECT COUNT(*) FROM Entry e WHERE e.Entry_Event_ID_FK = @P1) AS registrations_all,
            (SELECT COUNT(*) FROM Entry e WHERE e.Entry_Event_ID_FK = @P1 AND e.Entry_CancelValue > 0) AS registrations_cancelled,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Club_ID FROM Club c JOIN Entry e ON e.Entry_OwnerClub_ID_FK = c.Club_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_owner_clubs,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Crew_Athlete_ID_FK FROM Entry e JOIN Crew c ON c.Crew_Entry_ID_FK = e.Entry_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_athletes,
            (SELECT COUNT(*) FROM (SELECT DISTINCT c.Crew_Club_ID_FK FROM Entry e JOIN Crew c ON c.Crew_Entry_ID_FK = e.Entry_ID WHERE e.Entry_Event_ID_FK = @P1) AS count) AS registrations_clubs
          ",
        );
        query.bind(regatta_id);
        query
    }
}
