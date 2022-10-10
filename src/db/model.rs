use crate::db::utils::Column;
use serde::Serialize;
use std::time::Duration;
use tiberius::{time::chrono::NaiveDateTime, Row};

pub fn create_score(row: &Row) -> Score {
    Score {
        rank: Column::get(row, "rank"),
        club_short_label: Column::get(row, "Club_Abbr"),
        points: Column::get(row, "points"),
    }
}

pub fn create_regatta(row: &Row) -> Regatta {
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

pub fn create_heat(row: &Row) -> Heat {
    let date_time: NaiveDateTime = Column::get(row, "Comp_DateTime");

    Heat {
        id: Column::get(row, "Comp_ID"),
        race: create_race(row),
        number: Column::get(row, "Comp_Number"),
        round_code: Column::get(row, "Comp_RoundCode"),
        label: Column::get(row, "Comp_Label"),
        group_value: Column::get(row, "Comp_GroupValue"),
        state: Column::get(row, "Comp_State"),
        cancelled: Column::get(row, "Comp_Cancelled"),
        date: date_time.date().to_string(),
        time: date_time.time().to_string(),
        ac_num_sub_classes: Column::get(row, "AgeClass_NumSubClasses"),
        referee: create_referee(row),
    }
}

pub fn create_race(row: &Row) -> Race {
    let short_label: String = Column::get(row, "Offer_ShortLabel");
    let comment: String = Column::get(row, "Offer_Comment");
    Race {
        comment: comment.trim().to_owned(),
        number: Column::get(row, "Offer_RaceNumber"),
        short_label: short_label.trim().to_owned(),
        distance: Column::get(row, "Offer_Distance"),
    }
}

fn create_referee(row: &Row) -> Referee {
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

pub fn create_heat_registration(row: &Row) -> HeatRegistration {
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

    let result = HeatResult {
        delta,
        rank_label,
        rank_sort,
        result: Column::get(row, "Result_DisplayValue"),
        points: 0,
    };

    HeatRegistration {
        id: Column::get(row, "CE_ID"),
        lane: Column::get(row, "CE_Lane"),
        registration: create_registration(row),
        result,
    }
}

fn create_registration(row: &Row) -> Registration {
    let registration = Registration {
        bib: Column::get(row, "Entry_Bib"),
        comment: Column::get(row, "Entry_Comment"),
        boat_number: Column::get(row, "Entry_BoatNumber"),
        short_label: Column::get(row, "Label_Short"),
    };
    registration
}

#[derive(Debug, Serialize, Clone)]
pub struct Score {
    rank: i16,
    club_short_label: String,
    points: f64,
}

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

#[derive(Debug, Serialize, Clone)]
pub struct Registration {
    bib: i16,
    #[serde(rename = "boatNumber")]
    boat_number: i16,
    comment: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
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
    ac_num_sub_classes: u8,
    race: Race,
    referee: Referee,
}

#[derive(Debug, Serialize, Clone)]
pub struct HeatRegistration {
    pub id: i32,
    lane: i16,
    result: HeatResult,
    registration: Registration,
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

#[derive(Debug, Serialize, Clone, Default)]
pub struct Referee {
    #[serde(rename = "firstName")]
    first_name: String,
    #[serde(rename = "lastName")]
    last_name: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct Race {
    number: String,
    #[serde(rename = "shortLabel")]
    short_label: String,
    comment: String,
    distance: i16,
}
