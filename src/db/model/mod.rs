mod age_class;
mod athlete;
mod boat_class;
mod club;
mod column;
mod crew;
mod heat;
mod heat_registration;
mod heat_result;
mod race;
mod referee;
mod regatta;
mod registration;
mod score;
mod statistics;
mod utils;

pub use age_class::AgeClass;
pub use athlete::Athlete;
pub use boat_class::BoatClass;
pub use club::Club;
pub use column::Column;
pub use crew::Crew;
pub use heat::{Heat, Kiosk};
pub use heat_registration::HeatRegistration;
pub use heat_result::HeatResult;
pub use race::Race;
pub use referee::Referee;
pub use regatta::Regatta;
pub use registration::Registration;
pub use score::Score;
pub use statistics::Statistics;
use tiberius::{time::chrono::NaiveDateTime, Row};

pub trait ToEntity<T> {
    fn to_entity(&self) -> T;
}

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}

pub trait RowColumn<T> {
    fn get_column(&self, col_name: &str) -> T;
}

impl RowColumn<bool> for Row {
    fn get_column(&self, col_name: &str) -> bool {
        self.try_get::<bool, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<u8> for Row {
    fn get_column(&self, col_name: &str) -> u8 {
        self.try_get::<u8, _>(col_name).unwrap().unwrap_or_default()
    }
}

impl RowColumn<i16> for Row {
    fn get_column(&self, col_name: &str) -> i16 {
        self.try_get::<i16, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<i32> for Row {
    fn get_column(&self, col_name: &str) -> i32 {
        self.try_get::<i32, _>(col_name)
            .unwrap_or_default()
            .unwrap_or_default()
    }
}

impl RowColumn<f64> for Row {
    fn get_column(&self, col_name: &str) -> f64 {
        self.try_get::<f64, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<NaiveDateTime> for Row {
    fn get_column(&self, col_name: &str) -> NaiveDateTime {
        self.try_get::<NaiveDateTime, _>(col_name)
            .unwrap()
            .unwrap_or_default()
    }
}

impl RowColumn<String> for Row {
    fn get_column(&self, col_name: &str) -> String {
        self.try_get::<&str, _>(col_name)
            .unwrap()
            .unwrap_or_default()
            .to_string()
    }
}
