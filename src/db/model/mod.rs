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

pub trait RowToEntity<T> {
    fn to_entity(&self) -> T;
}

pub trait TryRowToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}
