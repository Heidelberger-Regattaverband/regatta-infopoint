mod age_class;
mod block;
mod boat_class;
mod heat_result;
mod referee;
mod regatta;
mod schedule;
pub mod utils;

pub use age_class::AgeClass;
pub use block::Block;
pub use boat_class::BoatClass;
pub use heat_result::HeatResult;
pub use referee::Referee;
pub use regatta::Regatta;
pub use schedule::{Schedule, ScheduleEntry};

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}
