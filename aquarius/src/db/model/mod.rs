mod age_class;
mod block;
mod boat_class;
pub mod utils;

pub use age_class::AgeClass;
pub use block::Block;
pub use boat_class::BoatClass;

pub trait TryToEntity<T> {
    fn try_to_entity(&self) -> Option<T>;
}
