// exposes sub-modules
pub mod aquarius;
pub mod pool;
pub mod utils;

use self::pool::TiberiusConnectionManager;
use bb8::Pool;

pub type TiberiusPool = Pool<TiberiusConnectionManager>;
