mod age_class;
mod athlete;
mod block;
mod boat_class;
mod club;
mod crew;
mod filters;
mod heat;
mod heat_registration;
mod heat_result;
mod race;
mod referee;
mod regatta;
mod schedule;
mod score;
mod statistics;
mod utils;

pub use age_class::AgeClass;
pub use athlete::Athlete;
pub use boat_class::BoatClass;
pub use club::Club;
pub use crew::Crew;
pub(crate) use filters::Filters;
pub use heat::{Heat, Kiosk};
pub use heat_registration::HeatRegistration;
pub use heat_result::HeatResult;
pub use race::Race;
pub use referee::Referee;
pub use regatta::Regatta;
pub use registration::Registration;
pub use score::Score;
pub use statistics::Statistics;
mod registration;
pub(crate) use schedule::Schedule;

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}
