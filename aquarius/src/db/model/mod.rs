mod age_class;
mod athlete;
mod block;
mod boat_class;
mod club;
mod crew;
mod filters;
mod heat_result;
mod referee;
mod regatta;
mod schedule;
mod score;
mod statistics;
pub mod utils;

pub use age_class::AgeClass;
pub use athlete::Athlete;
pub use block::Block;
pub use boat_class::BoatClass;
pub use club::Club;
pub use crew::Crew;
pub use filters::Filters;
pub use heat_result::HeatResult;
pub use referee::Referee;
pub use regatta::Regatta;
pub use schedule::{Schedule, ScheduleEntry};
pub use score::Score;
pub use statistics::Statistics;

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}
