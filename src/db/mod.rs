// exposes sub-modules
pub mod aquarius;
pub mod cache;
pub mod model;
pub mod pool;
pub mod utils;

use self::pool::TiberiusConnectionManager;

pub type TiberiusPool = bb8::Pool<TiberiusConnectionManager>;
